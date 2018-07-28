cargo build --release

$user = [Security.Principal.WindowsIdentity]::GetCurrent().Name

$action = New-ScheduledTaskAction -Execute (Resolve-Path .\target\release\gpg-restart-agent.exe)

$trigger1 = New-CimInstance -ClassName MSFT_TaskEventTrigger `
    -Namespace Root/Microsoft/Windows/TaskScheduler `
    -ClientOnly `
    -Property @{
    Enabled      = $true
    Subscription = @"
<QueryList>
    <Query Id="0" Path="Microsoft-Windows-SmartCard-DeviceEnum/Operational">
        <Select Path="Microsoft-Windows-SmartCard-DeviceEnum/Operational">
            *[System[Provider[@Name='Microsoft-Windows-SmartCard-DeviceEnum'] and EventID=101]]
        </Select>
    </Query>
    <Query Id="1" Path="System">
        <Select Path="System">
            *[System[Provider[@Name='Microsoft-Windows-Kernel-Power'] and EventID=107]]
        </Select>
    </Query>
</QueryList>
"@
}
$trigger2 = New-CimInstance -ClassName MSFT_TaskLogonTrigger `
    -Namespace Root/Microsoft/Windows/TaskScheduler `
    -ClientOnly `
    -Property @{
    Enabled = $true
    UserId  = $user
}

$principal = New-ScheduledTaskPrincipal -UserId $user

$task = New-ScheduledTask -Action $action -Principal $principal -Description "Restart gpg-agent when smartcard is inserted"
$task.triggers = @($trigger1, $trigger2)

$task | Register-ScheduledTask -TaskName "gpg-restart-agent"