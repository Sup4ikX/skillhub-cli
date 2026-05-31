param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:USERPROFILE\.skillhub\bin"
)

Write-Host "Installing skillhub..." -ForegroundColor Green

$Repo = "skillhub/skillhub"

# Detect architecture
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$Target = "x86_64-pc-windows-msvc"

# Fetch latest version if not specified
if ($Version -eq "latest") {
    try {
        $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
        $response = Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "skillhub-installer" }
        $Version = $response.tag_name
    } catch {
        Write-Warning "Failed to fetch latest version."
        Write-Warning "Try: powershell -c `".\install.ps1 -Version v0.1.0`""
        exit 1
    }
}

$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/skillhub-$Target.tar.gz"

Write-Host "  Platform:  $Target" -ForegroundColor Cyan
Write-Host "  Version:   $Version" -ForegroundColor Cyan
Write-Host "  Download:  $DownloadUrl" -ForegroundColor Cyan
Write-Host "  Install:   $InstallDir" -ForegroundColor Cyan
Write-Host ""

# Create install dir
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

# Download
$TmpDir = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null

try {
    $tmpFile = Join-Path $TmpDir "skillhub.tar.gz"
    Write-Host "Downloading..."
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $tmpFile

    Write-Host "Extracting..."
    tar xzf $tmpFile -C $TmpDir

    Copy-Item (Join-Path $TmpDir "skillhub.exe") (Join-Path $InstallDir "skillhub.exe") -Force

    Write-Host "Installed skillhub to $InstallDir\skillhub.exe" -ForegroundColor Green
} finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}

# Add to PATH
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallDir*") {
    $newPath = "$InstallDir;$currentPath"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "Added $InstallDir to PATH (user). Restart your terminal." -ForegroundColor Yellow
} else {
    Write-Host "PATH already configured." -ForegroundColor Gray
}

Write-Host ""
Write-Host "Run 'skillhub setup' to configure your GitHub token." -ForegroundColor Cyan
