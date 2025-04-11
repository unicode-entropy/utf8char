def main():
    for i in range(128, 256):
        # everything with 5 contiguous 1 bits is invalid
        if not i >= 0b1111_1000:
            print(f"    _{i} = 0b{i:b},")

if __name__ == "__main__":
    main()
