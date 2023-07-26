def start(os):
    paths = [
        ""
    ]

def main():
    systems = [
        "   1. Windows",
        "   2. Linux"
    ]

    print("Type the corresponding number to choose Operating System:")
    for sys in systems:
        print(sys)
    print("")

    os = None


    for i in range(0, 10):
        try:
            num = int(input("Select OS: "))
            if num > len(systems) or num < 1:
                print("Invalid input enter number in the range 1 to", len(systems))
            else:
                os = num
                break
        except:
            print("Invalid input enter number")

    if not os: 
        print("Retry attempts exceeded")
    else:
        start(os - 1)

if __name__ == "__main__":
    main()