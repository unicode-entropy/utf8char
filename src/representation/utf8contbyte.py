# appeases clippy by separating binary literals for clarity
def split(n: int) -> str:
    a = f"{n:b}"

    return f"{a[:4]}_{a[4:]}"


def main():
    for i in range(128, 256):
        if (i & 0b1100_0000) == 0b1000_0000:
            print(f"    _{i} = 0b{split(i)}, // {i:x}")


if __name__ == "__main__":
    main()
