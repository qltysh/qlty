# Licensed under the MIT license
# <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

<#
.SYNOPSIS

The installer for qlty (latest)

.DESCRIPTION

This script detects what platform you're on and fetches an appropriate archive from
https://qlty-releases.s3.amazonaws.com/qlty/latest
then unpacks the binaries and installs them to ($HOME\.qlty\bin)

It will then add that dir to PATH by editing your Environment.Path registry key

.PARAMETER InstallURL
The URL of the directory where artifacts can be fetched from

.PARAMETER NoModifyPath
Don't add the install directory to PATH

.PARAMETER Help
Print help

#>

param (
    [Parameter(HelpMessage = "The URL of the directory where artifacts can be fetched from")]
    [string]$InstallURL = 'https://qlty-releases.s3.amazonaws.com/qlty',
    [Parameter(HelpMessage = "Don't add the install directory to PATH")]
    [switch]$NoModifyPath,
    [Parameter(HelpMessage = "Print Help")]
    [switch]$Help
)

$Version = "latest"
if ($Env:QLTY_VERSION) {
    $Version = $Env:QLTY_VERSION
    $Version = $Version -replace "^v"
    $Version = "v$Version"
}

if ($Env:QLTY_NO_MODIFY_PATH) {
    $NoModifyPath = $true
}

if ($Env:QLTY_INSTALL_URL) {
    $InstallURL = $Env:QLTY_INSTALL_URL
}

$ArtifactDownloadUrl = "$InstallURL/$Version"

$app_name = 'qlty'
$app_version = "($Version)"

function Install-Binary($install_args) {
  if ($Help) {
    Get-Help $PSCommandPath -Detailed
    Exit
  }
  $old_erroractionpreference = $ErrorActionPreference
  $ErrorActionPreference = 'stop'

  Initialize-Environment

  # Platform info injected by qlty
  $platforms = @{
    "x86_64-pc-windows-msvc" = @{
      "artifact_name" = "qlty-x86_64-pc-windows-msvc.zip"
      "bins" = "qlty.exe"
      "zip_ext" = ".zip"
    }
  }

  $fetched = Download "$ArtifactDownloadUrl" $platforms
  Invoke-Installer $fetched "$install_args"

  $ErrorActionPreference = $old_erroractionpreference
}

function Get-TargetTriple() {
  try {
    # This might return X64 on ARM64 Windows, which is OK since emulation is available.
    # It works correctly starting in PowerShell Core 7.3 and Windows PowerShell in Win 11 22H2.
    # Ideally this would just be
    #   [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    # but that gets a type from the wrong assembly on Windows PowerShell (i.e. not Core)
    $a = [System.Reflection.Assembly]::LoadWithPartialName("System.Runtime.InteropServices.RuntimeInformation")
    $t = $a.GetType("System.Runtime.InteropServices.RuntimeInformation")
    $p = $t.GetProperty("OSArchitecture")
    # Possible OSArchitecture Values: https://learn.microsoft.com/dotnet/api/system.runtime.interopservices.architecture
    # Rust supported platforms: https://doc.rust-lang.org/stable/rustc/platform-support.html
    switch ($p.GetValue($null).ToString())
    {
      "X86" { return "i686-pc-windows-msvc" }
      "X64" { return "x86_64-pc-windows-msvc" }
      "Arm" { return "thumbv7a-pc-windows-msvc" }
      "Arm64" { return "aarch64-pc-windows-msvc" }
    }
  } catch {
    # The above was added in .NET 4.7.1, so Windows PowerShell in versions of Windows
    # prior to Windows 10 v1709 may not have this API.
    Write-Verbose "Get-TargetTriple: Exception when trying to determine OS architecture."
    Write-Verbose $_
  }

  # This is available in .NET 4.0. We already checked for PS 5, which requires .NET 4.5.
  Write-Verbose("Get-TargetTriple: falling back to Is64BitOperatingSystem.")
  if ([System.Environment]::Is64BitOperatingSystem) {
    return "x86_64-pc-windows-msvc"
  } else {
    return "i686-pc-windows-msvc"
  }
}

function Download($download_url, $platforms) {
  $arch = Get-TargetTriple

  if (-not $platforms.ContainsKey($arch)) {
    # X64 is well-supported, including in emulation on ARM64
    Write-Verbose "$arch is not availablem falling back to X64"
    $arch = "x86_64-pc-windows-msvc"
  }

  if (-not $platforms.ContainsKey($arch)) {
    # should not be possible, as currently we always produce X64 binaries.
    $platforms_json = ConvertTo-Json $platforms
    throw "ERROR: could not find binaries for this platform. Last platform tried: $arch platform info: $platforms_json"
  }

  # Lookup what we expect this platform to look like
  $info = $platforms[$arch]
  $zip_ext = $info["zip_ext"]
  $bin_names = $info["bins"]
  $artifact_name = $info["artifact_name"]

  # Make a new temp dir to unpack things to
  $tmp = New-Temp-Dir
  $dir_path = "$tmp\$app_name$zip_ext"

  # Download and unpack!
  $url = "$download_url/$artifact_name"
  Write-Information "Downloading $app_name $app_version ($arch)"
  Write-Verbose "  from $url"
  Write-Verbose "  to $dir_path"
  $wc = New-Object Net.Webclient
  $wc.downloadFile($url, $dir_path)

  Write-Verbose "Unpacking to $tmp"

  # Select the tool to unpack the files with.
  #
  # As of windows 10(?), powershell comes with tar preinstalled, but in practice
  # it only seems to support .tar.gz, and not xz/zstd. Still, we should try to
  # forward all tars to it in case the user has a machine that can handle it!
  switch -Wildcard ($zip_ext) {
    ".zip" {
      Expand-Archive -Path $dir_path -DestinationPath "$tmp";
      Break
    }
    ".tar.*" {
      tar xf $dir_path --strip-components 1 -C "$tmp";
      Break
    }
    Default {
      throw "ERROR: unknown archive format $zip_ext"
    }
  }

  # Let the next step know what to copy
  $bin_paths = @()
  foreach ($bin_name in $bin_names) {
    Write-Verbose "  Unpacked $bin_name"
    $bin_paths += "$tmp\$bin_name"
  }
  return $bin_paths
}

function Invoke-Installer($bin_paths) {

  $dest_dir = if (($base_dir = $HOME)) {
    Join-Path $base_dir ".qlty\bin"
  } else {
    throw "ERROR: could not find your HOME dir to install binaries to"
  }

  if ($Env:QLTY_INSTALL_BIN_PATH) {
    $dest_dir = $Env:QLTY_INSTALL_BIN_PATH
  }

  $dest_dir = New-Item -Force -ItemType Directory -Path $dest_dir
  Write-Information "Installing to $dest_dir"
  # Just copy the binaries from the temp location to the install dir
  foreach ($bin_path in $bin_paths) {
    $installed_file = Split-Path -Path "$bin_path" -Leaf
    if (Test-Path "$dest_dir\$installed_file") {
        try {
            Remove-Item "$dest_dir\$installed_file.prev" -Force
        } catch { }
        Rename-Item "$dest_dir\$installed_file" -NewName "$dest_dir\$installed_file.prev"
    }
    Copy-Item "$bin_path" -Destination "$dest_dir"
    Remove-Item "$bin_path" -Recurse -Force
    try {
        Remove-Item "$dest_dir\$installed_file.prev" -Force 2>$null
    } catch { }
    Write-Information "  $installed_file"
  }

  # Transparent support for GitHub CI
  if ($Env:GITHUB_PATH) {
    Add-Content -Path $Env:GITHUB_PATH -Value "$dest_dir"
  }

  Write-Information "Everything's installed!"
  if (-not $NoModifyPath) {
    if (Add-Path $dest_dir) {
        Write-Information ""
        Write-Information "$dest_dir was added to your PATH, you may need to restart your shell for that to take effect."
    }
  }
}

# Try to add the given path to PATH via the registry
#
# Returns true if the registry was modified, otherwise returns false
# (indicating it was already on PATH)
function Add-Path($OrigPathToAdd) {
  $RegistryPath = "HKCU:\Environment"
  $PropertyName = "Path"
  $PathToAdd = $OrigPathToAdd

  $Item = if (Test-Path $RegistryPath) {
    # If the registry key exists, get it
    Get-Item -Path $RegistryPath
  } else {
    # If the registry key doesn't exist, create it
    Write-Verbose  "Creating $RegistryPath"
    New-Item -Path $RegistryPath -Force
  }

  $OldPath = ""
  try {
    # Try to get the old PATH value. If that fails, assume we're making it from scratch.
    # Otherwise assume there's already paths in here and use a ; separator
    $OldPath = $Item | Get-ItemPropertyValue -Name $PropertyName
    $PathToAdd = "$PathToAdd;"
  } catch {
    # We'll be creating the PATH from scratch
    Write-Verbose "Adding $PropertyName Property to $RegistryPath"
  }

  # Check if the path is already there
  #
  # We don't want to incorrectly match "C:\blah\" to "C:\blah\blah\", so we include the semicolon
  # delimiters when searching, ensuring exact matches. To avoid corner cases we add semicolons to
  # both sides of the input, allowing us to pretend we're always in the middle of a list.
  if (";$OldPath;" -like "*;$OrigPathToAdd;*") {
    # Already on path, nothing to do
    Write-Verbose "install dir already on PATH, all done!"
    return $false
  } else {
    # Actually update PATH
    Write-Verbose "Adding $OrigPathToAdd to your PATH"
    $NewPath = $PathToAdd + $OldPath
    # We use -Force here to make the value already existing not be an error
    $Item | New-ItemProperty -Name $PropertyName -Value $NewPath -PropertyType String -Force | Out-Null
    return $true
  }
}

function Initialize-Environment() {
  If (($PSVersionTable.PSVersion.Major) -lt 5) {
    Write-Error "PowerShell 5 or later is required to install $app_name."
    Write-Error "Upgrade PowerShell: https://docs.microsoft.com/en-us/powershell/scripting/setup/installing-windows-powershell"
    break
  }

  # show notification to change execution policy:
  $allowedExecutionPolicy = @('Unrestricted', 'RemoteSigned', 'ByPass')
  If ((Get-ExecutionPolicy).ToString() -notin $allowedExecutionPolicy) {
    Write-Error "PowerShell requires an execution policy in [$($allowedExecutionPolicy -join ", ")] to run $app_name."
    Write-Error "For example, to set the execution policy to 'RemoteSigned' please run :"
    Write-Error "'Set-ExecutionPolicy RemoteSigned -scope CurrentUser'"
    break
  }

  # GitHub requires TLS 1.2
  If ([System.Enum]::GetNames([System.Net.SecurityProtocolType]) -notcontains 'Tls12') {
    Write-Error "Installing $app_name requires at least .NET Framework 4.5"
    Write-Error "Please download and install it first:"
    Write-Error "https://www.microsoft.com/net/download"
    break
  }
}

function New-Temp-Dir() {
  [CmdletBinding(SupportsShouldProcess)]
  param()
  $parent = [System.IO.Path]::GetTempPath()
  [string] $name = [System.Guid]::NewGuid()
  New-Item -ItemType Directory -Path (Join-Path $parent $name)
}

function Send-Installed-Event {
    $event_payload = @{
        event = "CLI Installed"
        properties = @{
            Target = $target
            Environment = $Env:AWS_EXECUTION_ENV -or "LOCAL"
            CI = $Env:CI -or "false"
            Installer = "install.ps1"
        }
        anonymousId = "ef2de8ab98bd4b85d36e62e7323345b2"
    } | ConvertTo-Json

    $headers = @{
        Authorization = "Basic MTc2YzFjYzYyYTdmN2UzOTczMDI6"
        "Content-Type" = "application/json"
    }

    try {
        Invoke-RestMethod -Uri "https://cdp.customer.io/v1/track" -Method Post -Headers $headers -Body $event_payload >$null
    } catch {
        Write-Error "Failed to send installed event: $_"
    }
}

# PSScriptAnalyzer doesn't like how we use our params as globals, this calms it
$Null = $ArtifactDownloadUrl, $NoModifyPath, $Help
# Make Write-Information statements be visible
$InformationPreference = "Continue"
Install-Binary "$Args"
Send-Installed-Event
