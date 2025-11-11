# --------- CAPPUCINA DETECT_OS.py --------- #
# Used for easy operating system detection.
# Under the MIT license, to see more please refer to LICENSE.md
# --------- WRITTEN BY @NVTTLES --------- #
import os

family = os.name

def check_os():
    match family:
        case "posix":
            return "posix"
        case "nt":
            return "nt"
        case "java":
            return "java"

def main():
    print(str(check_os()))

if __name__ == "__main__":
    main()