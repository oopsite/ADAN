import os
import platform

def check_os() -> str:
    system_name = platform.system().lower() # fetch the kernel you're using
    os_mapping = {
        "linux": "posix",
        "darwin": "posix", # macOS
        "windows": "nt",
        "java": "java"
    }

    return os_mapping.get(system_name, "Unknown")