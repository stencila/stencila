# Stencila + Vagrant

[Vagrant](https://www.vagrantup.com/) is a virtual machine (VM) manager which eases the configuration and provisioning of VMs running on [VirtualBox](https://www.virtualbox.org/) (or other provider). We use Vagrant to create virtual machines for building Stencila packages for multiple operating systems. 

This Vagrantfile defines configurations for several platform-specific VMs. For each platform the VM is setup using one of the scripts in the `../setup` directory. We use shell scripts, rather than other provisioning tools (e.g. Ansible), because the setup is relatively simple.

Once VirtualBox and Vagrant are installed on the host machine launch one of the platform VMs using `vagrant up` in this directory e.g. 

    vagrant up ubuntu-14.04-32-build
 
Then SSH into it,
 
    vagrant ssh ubuntu-14.04-32-build
 
Change into the VM's share of this directory, called `/vagrant`, and run `Makefile` recipes e.g.
 
    cd /vagrant
    make cpp-package py-package r-package

When finished grab the built packages from the `build` directory.

For better performance it is recommended to use a build directory which is on the VM, instead of the default `stencila/build/OS/ARCH/VERSION` directory within a shared folder on the host. For example, with Linux:

    # Create build directory on the guest VM
    mkdir -p ~/build
    # Change into the top level `stencila` directory on the host (mapped to `/vagrant`)
    cd /vagrant
    # Specify the build directory when invoking `make`
    make py-package BUILD=~/build

When finished, shutdown the VM with `halt`:

    vagrant halt ubuntu-14.04-32-build

There are also Vagrant machines defined for testing the installation of Stencila packages on user machines. These have a reduced provisioning which only includes the bare necessities for installation.
