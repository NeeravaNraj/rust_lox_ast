import os
import sys
import shutil

def check_exists(path):
    if not os.path.exists(path):
        print(f"'{path}' directory does not exist!", file=sys.stderr)
        exit(-1)
    if not os.path.isdir(path):
        print(f"'{path}' is not a directory!", file=sys.stderr)
        exit(-1)

def start(o_s):
    paths = [
        "~\.vscode\extensions",
        "~/.vscode/extensions"
        "~/.vscode/extensions"
    ]
    extensions_path = os.path.expanduser(paths[o_s])
    check_exists(extensions_path)
    lox_highlighting = os.path.abspath("./lox_highlighting")
    check_exists(lox_highlighting)
    extensions_path = os.path.join(extensions_path, "lox")

    if os.path.exists(extensions_path):
        print("Files already exist!")
        overwrite = None
        for _ in range(0, 10):
            data = input("Would you like to overwrite them (y/n) ")
            if data == "y":
                overwrite = data
                break
            elif data == "n":
                return
            else:
                print("Invalid input type either 'y' or 'n'", file=sys.stderr)
        if overwrite:
            shutil.rmtree(extensions_path)
    print("Copying files!")
    shutil.copytree(lox_highlighting, extensions_path)

def main():
    systems = [
        "   1. Windows",
        "   2. Linux",
        "   3. MacOS"
    ]

    print("Type the corresponding number to choose Operating System:")
    for sys in systems:
        print(sys)
    print("")

    o_s = None


    for _ in range(0, 10):
        try:
            num = int(input("Select OS: "))
            if num > len(systems) or num < 1:
                print("Invalid input enter number in the range 1 to", len(systems))
            else:
                o_s = num
                break
        except:
            print("Invalid input enter number")

    if not o_s: 
        print("Retry attempts exceeded")
    else:
        start(o_s - 1)
        print("Done.")

if __name__ == "__main__":
    main()
