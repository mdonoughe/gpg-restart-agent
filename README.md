# gpg-restart-agent

This is a small program that uses gpg-connect-agent to (re)start gpg-agent, but it uses the Windows subsystem and launches gpg-connect-agent with CREATE_NO_WINDOW set, meaning it does not flash anything on the screen when it does it.

Use install.ps1 to make it execute when you log in, when you insert a smartcard, and when the computer wakes from sleep.

To uninstall, delete the scheduled task:

    Get-ScheduledTask -TaskPath '\' -TaskName 'gpg-restart-agent' | Unregister-ScheduledTask
