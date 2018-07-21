cargo build --release

$action = New-ScheduledTaskAction -Execute (Resolve-Path .\target\release\gpg-restart-agent.exe)

$trigger = New-CimInstance -ClassName MSFT_TaskEventTrigger `
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
</QueryList>
"@
}

$user = [Security.Principal.WindowsIdentity]::GetCurrent().Name
$principal = New-ScheduledTaskPrincipal -UserId $user

$task = New-ScheduledTask -Action $action -Principal $principal -Description "Restart gpg-agent when smartcard is inserted"
$task.triggers = @($trigger)

$task | Register-ScheduledTask -TaskName "gpg-restart-agent"