# Vagrantfile for building and testing Stencila packages for various platforms.
# 
# We use [Vagrant](https://www.vagrantup.com/) to create virtual machines
# for building Stencila packages for multiple operating systems. 
# Vagrant is a virtual machine (VM) manager
# which eases the configuration and provisioning of VMs running 
# on [VirtualBox](https://www.virtualbox.org/) (or other).
# 
# This Vagrantfile defines the configuration for each platform VM.
# For each platform there is a provisioning script in this directory, starting
# with `provision-`. We use shell scripts, rather than other provisioning 
# tools (e.g. Ansible), because the provisioning is relatively simple.
# 
# Once VirtualBox and Vagrant are installed on the host machine launch one of the 
# platform VMs using `vagrant up` in this directory e.g. 
# 
#    vagrant up ubuntu-14.04-32-build
#  
# Then SSH into it,
#  
#    vagrant ssh ubuntu-14.04-32-build
#  
# Change into the VM's share of this directory, called `/vagrant`, and run
# a task in the `Makefile` e.g.
#  
#    cd /vagrant
#    make cpp-package py-package r-package
#  
# This will create a new directory in the `build` subdirectory for the platform.
# 
# When finished shutdown the VM with `halt`:
# 
#    vagrant halt ubuntu-14.04-32-build
#
# When using a VM, for better performance it is recommended to use a build 
# directory which is on the VM, instead of the default `stencila/build/OS/ARCH/VERSION` 
# directory within a shared folder on the host. For example, with Linux:
#
# Create build directory on the guest VM
#    mkdir -p ~/build
# Change into the `stencila` directory on the host (mapped to `/vagrant`)
#    cd /vagrant
# Specify the build directory when invoking `make`
#    make cpp-package r-package py-package BUILD=~/build
#
# There are also Vagrant machines defined for testing the installation of Stencila
# packages on user machines. These have a reduced provisioning which only includes
# the bare necessities for installation.

Vagrant.configure("2") do |config|

    # For building on Ubuntu 12.04 LTS (Precise Pangolin) 64 bit
    # This is a similar VM to that used on Travis CI (http://docs.travis-ci.com/user/ci-environment/)
    config.vm.define "ubuntu-12.04-64-build" do |platform|
        platform.vm.box = "ubuntu/precise64"
        platform.vm.provision "shell", path: "provision-ubuntu-12.04-build.sh"
        platform.vm.provider "virtualbox" do |provider|
            provider.name = "stencila-ubuntu-12.04-64-build"
            provider.memory = 3072
        end
    end

    # For building on Ubuntu 14.04 LTS (Trusty Tahr) 32 bit
    config.vm.define "ubuntu-14.04-32-build" do |platform|
        platform.vm.box = "ubuntu/trusty32"
        platform.vm.provision "shell", path: "provision-ubuntu-14.04-build.sh"
        platform.vm.provider "virtualbox" do |provider|
            provider.name = "stencila-ubuntu-14.04-32-build"
            provider.memory = 1024
        end
    end

    # For building on Ubuntu 14.04 LTS (Trusty Tahr) 64 bit
    config.vm.define "ubuntu-14.04-64-build" do |platform|
        platform.vm.box = "ubuntu/trusty64"
        platform.vm.provision "shell", path: "provision-ubuntu-14.04-build.sh"
        platform.vm.provider "virtualbox" do |provider|
            provider.name = "stencila-ubuntu-14.04-64-build"
            provider.memory = 1024
        end
    end

    # For testing use on Ubuntu 14.04 LTS (Trusty Tahr) 64 bit
    config.vm.define "ubuntu-14.04-64-use" do |platform|
        platform.vm.box = "ubuntu/trusty64"
        platform.vm.provision "shell", path: "provision-ubuntu-14.04-use.sh"
        platform.vm.network "forwarded_port", guest: 7373, host: 7374
        platform.vm.provider "virtualbox" do |provider|
            provider.name = "stencila-ubuntu-14.04-64-use"
            provider.memory = 1024
        end
    end

    # For testing use on Ubuntu 14.10 (Utopic Unicorn) 64 bit
    config.vm.define "ubuntu-14.10-64-use" do |platform|
        platform.vm.box = "larryli/utopic64"
        platform.vm.provision "shell", path: "provision-ubuntu-14.10-use.sh"
        platform.vm.network "forwarded_port", guest: 7373, host: 7374
        platform.vm.provider "virtualbox" do |provider|
            provider.name = "stencila-ubuntu-14.10-64-use"
            provider.memory = 1024
        end
    end

end
