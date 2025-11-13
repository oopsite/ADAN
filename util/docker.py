import subprocess
import sys
from detect_os import check_os

def is_docker_installed():
    try:
        subprocess.run(["docker", "--version"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        print("Docker is already installed on your system.")
        return True
    except subprocess.CalledProcessError:
        return False

def install_chocolatey():
    print("Installing Chocolatey package manager...")
    install_command = (
        "Set-ExecutionPolicy Bypass -Scope Process -Force; "
        "iwr https://community.chocolatey.org/install.ps1 -UseBasicPInvoke | iex"
    )

    try:
        subprocess.run(["powershell", "-Command", install_command], check=True)
        print("Chocolatey installed successfully.")
    except subprocess.CalledProcessError as e:
        print(f"Error installing Chocolatey: {e}")
        sys.exit(1)

def install_docker_windows():
    print("Installing Docker with Chocolatey...")
    install_command = "choco install docker-desktop -y"

    try:
        subprocess.run(["powershell", "-Command", install_command], check=True)
        print("Docker Desktop installation started. It may take a few minutes.")
    except subprocess.CalledProcessError as e:
        print(f"Error installing Docker: {e}")
        sys.exit(1)

def install_docker():
    if is_docker_installed() == True:
        return

    kernel = check_os()
    if kernel == "posix":
        if sys.platform.startswith("linux"):
            subprocess.run([
                "curl", "-fsSL", "https://download.docker.com/linux/static/stable/x86_64/docker-20.10.9.tgz", "-o", "/tmp/docker.tgz"
            ], check=True)
            subprocess.run(["sudo", "tar", "--strip-components=1", "-xvzf", "/tmp/docker.tgz", "-C", "/usr/local/bin"], check=True)
            subprocess.run(["rm", "/tmp/docker.tgz"], check=True)
            subprocess.run(["sudo", "chmod", "+x", "/usr/local/bin/docker"], check=True)
        elif sys.platform == "darwin":
            subprocess.run([
                "curl", "-fsSL", "https://download.docker.com/mac/stable/Docker.dmg", "-o", "/tmp/Docker.dmg"
            ], check=True)
            subprocess.run(["hdiutil", "attach", "/tmp/Docker.dmg"], check=True)
            subprocess.run(["sudo", "cp", "-R", "/Volumes/Docker/Docker.app", "/Applications/"], check=True)
            subprocess.run(["hdiutil", "detach", "/Volumes/Docker"], check=True)
        print("Docker has been installed on your system, exiting")
    elif kernel == "nt":
        # print("Please install Docker for Windows from https://www.docker.com/products/docker-desktop")
        install_docker_windows()
    else:
        print("Unsupported OS for Docker installation.")

install_docker()