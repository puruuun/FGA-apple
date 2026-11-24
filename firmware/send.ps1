[CmdletBinding(PositionalBinding = $false)]
param(
    [Parameter(Mandatory = $true, Position = 0, ValueFromRemainingArguments = $true)]
    [string[]]$Commands,
    [string]$Port = 'COM3',
    [int]$WaitMs = 1500
)

$sp = New-Object System.IO.Ports.SerialPort $Port, 115200
$sp.DtrEnable = $true
$sp.RtsEnable = $false
$sp.ReadTimeout = 200
$sp.NewLine = "`n"
$sp.Open()
try {
    Start-Sleep -Milliseconds 300
    $sp.DiscardInBuffer()
    foreach ($c in $Commands) {
        $sp.WriteLine($c)
        $deadline = (Get-Date).AddMilliseconds($WaitMs)
        while ((Get-Date) -lt $deadline) {
            try {
                $line = $sp.ReadLine().Trim()
                if ($line) {
                    "$c -> $line"
                    if ($line -like 'OK*' -or $line -like 'ERR*') { break }
                }
            } catch [System.TimeoutException] { }
        }
    }
} finally {
    $sp.Close()
}
