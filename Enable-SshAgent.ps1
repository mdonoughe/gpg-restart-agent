<#
.SYNOPSIS

Configures gpg-agent to support SSH.

.NOTES

You will likely need to restart the agent after this.

#>
[CmdletBinding()]
param()

"enable-ssh-support:0:1`nenable-putty-support:0:1" | gpgconf --change-options gpg-agent
