from typing import Callable

filters: list[tuple[Callable[[int], bool], str]] = [
    (lambda i: i >= 0b1111_1000, "everything with 5 contiguous 1 bits is invalid"),
    (lambda i: (i & 0b1100_0000) == 0b1000_0000, "unicode cont bytes are invalid"),
    (
        lambda i: i in {0b1100_0000, 0b1100_0001},
        "guaranteed overlong encodings are invalid",
    ),
]


# appeases clippy by separating binary literals for clarity
def split(n: int) -> str:
    a = f"{n:b}"

    return f"{a[:4]}_{a[4:]}"


def main():
    for i in range(128, 256):
        valid = True

        for f in filters:
            matches, err = f

            if matches(i):
                print(f"    // SKIP: _{i} = 0b{split(i)}, {err}")
                valid = False
                break

        if valid:
            print(f"    _{i} = 0b{split(i)}, // {i:x}")


if __name__ == "__main__":
    main()
