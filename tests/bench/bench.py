import math
import os
from subprocess import DEVNULL, call
from time import time

N_RUNS = 1000

mean = lambda x: sum(x) / len(x)
std = lambda x: math.sqrt(sum(map(lambda y: (y - mean(x))**2, x)) / len(x))
se = lambda x: std(x) / math.sqrt(len(x))

def format(vals):
    vals_mean = mean(vals)
    vals_se = se(vals)
    valid_decimals = int(1 - math.log10(vals_se))
    return f"{round(vals_mean, valid_decimals)}s (±{round(vals_se, valid_decimals)} - {N_RUNS} runs)"

bench_dir = os.path.dirname(os.path.abspath(__file__))
lyt_kit = os.path.join(bench_dir, "lyt_kit")

times = []
for _ in range(N_RUNS):
    start = time()
    call(["mdzk", lyt_kit], stdout=DEVNULL)
    end = time()
    times.append(end - start)

print(f"Time taken: {format(times)}")
