# Vagrantfile for building Stencila library modules for various platforms.
# 
# We use [Vagrant](https://www.vagrantup.com/) to create virtual machines
# for building Stencila library modules. Vagrant is a virtual machine (VM) manager
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
#    vagrant up ubuntu-14.04-32
#  
# Then SSH into it,
#  
#    vagrant ssh ubuntu-14.04-32
#  
# Change into the VM's share of this directory, called `/vargrant`, and run
# a task in the `Makefile` e.g.
#  
#    cd /vagrant
#    make py-package r-package
#  
# This will create a new directory in the `build` subdirecoty for the platform.
# 
# When finished shutdown the VM with `halt`:
# 
#    vagrant halt ubuntu-14.04-32

Vagrant.configure("2") do |config|

    config.vm.define "ubuntu-14.04-32" do |platform|
        platform.vm.box = "ubuntu/trusty32"
        platform.vm.provision "shell", path: "provision-ubuntu-14.04.sh"
        platform.vm.provider "virtualbox" do |provider|
            provider.name = "stencila-ubuntu-14.04-32"
            provider.memory = 1024
        end
    end

    config.vm.define "ubuntu-14.04-64" do |platform|
        platform.vm.box = "ubuntu/trusty64"
        platform.vm.provision "shell", path: "provision-ubuntu-14.04.sh"
        platform.vm.provider "virtualbox" do |provider|
            provider.name = "stencila-ubuntu-14.04-64"
            provider.memory = 1024
        end
    end

end
