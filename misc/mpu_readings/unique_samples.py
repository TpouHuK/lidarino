def process_file(filename):
    with open(filename) as f:
        prev_values = None
        unique_values = 0
        total = 0
        duration = 0
        for line in f.readlines():
            total += 1
            x, y, z, d = map(float, line.split())
            if (x, y, z) != prev_values:
                unique_values += 1
            duration += d
            prev_values = (x, y, z)

    print(f"{filename} got {unique_values} unique_values out of {total}")
    print(f"{filename} took {duration} seconds")
    print(f"{filename} has {unique_values / duration} per second")


filenames = ["fchoice0", "max_dlpf", "dlpf_8khz", "dlpf_1khz", "dlpf_0"]
for n in filenames:
    process_file(n)
