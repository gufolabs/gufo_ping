# ---------------------------------------------------------------------
# Gufo Ping: Fast parallel ping example
# ---------------------------------------------------------------------
# Copyright (C) 2022, Gufo Labs
# ---------------------------------------------------------------------

import sys
import asyncio
import time
from multiprocessing import cpu_count
from threading import Thread
from queue import Queue
from typing import List
from gufo.ping import Ping

# Maximal amounts of CPU used
MAX_CPU = 128
# Number of worker tasks within every thread
N_TASKS = 50


def main(path: str) -> None:
    """
    Main pinger function.

    Args:
        path: Path to the list of IP addresses,
            each address on its own line
    """
    # Read file
    with open(path) as f:
        data = [x.strip() for x in f.readlines() if x.strip()]
    # Get effective CPUs
    n_data = len(data)
    # Effective number of workers cannot be more than
    # * amount of addresses to ping
    # * available CPUs
    # * Imposed CPU limit
    n_workers = min(MAX_CPU, cpu_count(), n_data)
    # Queue to pull the results
    result_queue = Queue()
    # Create worker threads and pass the shards of data
    workers = [
        Thread(
            target=worker,
            args=(data[n::n_workers], result_queue),
            name=f"worker-{n}",
        )
        for n in range(n_workers)
    ]
    # Collect starting time
    t0 = time.time()
    # Run threads
    for w in workers:
        w.start()
    # Retrieve measured results back
    success = 0
    for _ in range(n_data):
        addr, rtt = result_queue.get()
        if rtt is None:
            print(f"{addr}: timed out")
        else:
            print(f"{addr}: {rtt * 1000.0:.3f}ms")
            success += 1
    # Report performance
    dt = time.time() - t0
    print(f"--- {success} ok, {n_data - success} failed")
    print(
        f"--- {n_data} addresses, {dt:.3f}s, {float(n_data) / dt:.1f} addr/sec"
    )
    # Wait until all worker threads terminated properly
    for w in workers:
        w.join()


def worker(data: List[str], result_queue: Queue) -> None:
    """
    Thread worker, started within every thread.

    Args:
        data: List of IP addresses to ping.
        resut_queue: Queue to push results back.
    """
    # Create separate event loop per each thread
    loop = asyncio.new_event_loop()
    # And set it as default
    asyncio.set_event_loop(loop)
    # Run asynchronous worker within every thread
    loop.run_until_complete(async_worker(data, result_queue))
    # Cleanup
    loop.close()


async def async_worker(data: List[str], result_queue: Queue) -> None:
    """
    Asynchronous worker. Started for each thread.

    Args:
        data: List of IP addresses to ping.
        resut_queue: Queue to push results back.
    """

    async def task(addr_queue: asyncio.Queue, done: asyncio.Event):
        """
        Worker task. Up to N_TASKS spawn per thread.

        Args:
            addr_queue: Queue to pull addresses to ping. Stops when
                pulled None.
            done: Event to set when processing complete.
        """
        while True:
            # Pull address or None
            addr = await addr_queue.get()
            if not addr:
                # Stop on None
                break
            # Send ping and await the result
            rtt = await ping.ping(addr)
            # Push measured result to a main thread
            result_queue.put((addr, rtt))
        # Report worker is stopped.
        done.set()

    # Create ping socket per each thread
    ping = Ping()
    # Address queue
    addr_queue = asyncio.Queue(maxsize=2 * N_TASKS)
    # List of events to check tasks is finished
    finished = []
    # Effective tasks is limited by:
    # * Available addresses
    # * Imposed limit
    n_tasks = min(len(data), N_TASKS)
    # Create and run tasks
    loop = asyncio.get_running_loop()
    for _ in range(n_tasks):
        cond = asyncio.Event()
        loop.create_task(task(addr_queue, cond))
        finished.append(cond)
    # Push data to address queue,
    # may be blocked if we'd pushed too many
    for x in data:
        await addr_queue.put(x)
    # Push stopping None for each task
    for _ in range(n_tasks):
        await addr_queue.put(None)
    # Wait for each task to complete
    for cond in finished:
        await cond.wait()


if __name__ == "__main__":
    main(sys.argv[1])
