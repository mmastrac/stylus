# SSH Monitoring

SSH monitoring allows you to execute commands on remote systems and check their status. This is useful for monitoring servers, network devices, and other systems that support SSH access.

## SSH Configuration

To make your life easier, you can collect all of your SSH credentials in [a configuration file](https://www.ssh.com/ssh/config/). The examples in this section will assume you've got a central SSH configuration file.

```
host pi-*
	User matt
host tower
	User root
host unifi-*
	User admin
host *
	IdentityFile /srv/ssh_id_rsa
```

## Security Considerations

Depending on your security requirements, you may wish to loosen some of your SSH client's security check requirements. By disabling strict host key checking and host IP checking, your monitors will be more reliable but there will be [some tradeoffs](https://security.stackexchange.com/questions/161520/what-is-the-actual-drawback-of-checkhostip-no).

```bash
ssh <...> -oStrictHostKeyChecking=no -oCheckHostIP=no
```

## Basic SSH Check

```bash
# Assumes that `ssh_config` lives in the same folder as this script 
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
SSH_CONFIG=$DIR/ssh_config

ssh_check () {
    local host="$STYLUS_MONITOR_ID"
    ssh -F $SSH_CONFIG $host -oStrictHostKeyChecking=no -oCheckHostIP=no "true"
}

ssh_check
```

## SSH + Gather Basic Hardware Info

```bash
# Assumes that `ssh_config` lives in the same folder as this script 
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
SSH_CONFIG=$DIR/ssh_config

ssh_check () {
    local host="$STYLUS_MONITOR_ID"
    ssh -F $SSH_CONFIG $host -oStrictHostKeyChecking=no -oCheckHostIP=no \
        "uname -a && uptime && cat /proc/cpuinfo | grep -i -E '(hardware|model|stepping|revision)' | sort | uniq"
}

ssh_check
```

## Alternatives

For simpler connectivity tests, consider [ping monitoring](../ping/). For network devices, consider [SNMP monitoring](../snmp/). 