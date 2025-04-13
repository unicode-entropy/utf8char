def main():
    for i in range(128, 256):
        if (i & 0b1100_0000) == 0b1000_0000:
            print(f"    _{i} = 0b{i:b}, // {i:x}")


if __name__ == "__main__":
    main()
