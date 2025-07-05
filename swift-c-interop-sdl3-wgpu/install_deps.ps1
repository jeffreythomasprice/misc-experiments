# Remove-Item .deps -Recurse -Force
# Remove-Item SDL3.lib
# Remove-Item SDL3.dll

function DownloadHelper {
	param (
		$URL,
		$OutFile
	)
	if (Test-Path($OutFile)) {
		Write-Host "$OutFile already exists, skipping download" -ForegroundColor Yellow
	} else {
		Invoke-WebRequest $URL -OutFile $OutFile
		if ($OutFile -match '.zip$') {
			Expand-Archive -path $OutFile
		}
	}
}

New-Item -ItemType Directory -Force -Path .deps
cd .deps

DownloadHelper -URL "https://github.com/libsdl-org/SDL/releases/download/release-3.2.16/SDL3-devel-3.2.16-VC.zip" -OutFile SDL3.zip
Copy-Item -Path "SDL3/SDL3-3.2.16/include/SDL3" -Destination "../Sources/CSDL/" -Recurse -Force
Copy-Item -Path "SDL3/SDL3-3.2.16/lib/x64/SDL3.lib" -Destination "../" -Recurse -Force
Copy-Item -Path "SDL3/SDL3-3.2.16/lib/x64/SDL3.dll" -Destination "../" -Recurse -Force

DownloadHelper -URL "https://github.com/gfx-rs/wgpu-native/releases/download/v25.0.2.1/wgpu-windows-x86_64-msvc-release.zip" -OutFile wgpu-windows-x86_64-msvc-release.zip
Copy-Item -Path "wgpu-windows-x86_64-msvc-release/include/webgpu" -Destination "../Sources/CWGPU/" -Recurse -Force
Copy-Item -Path "wgpu-windows-x86_64-msvc-release/lib/wgpu_native.lib" -Destination "../" -Recurse -Force

cd ..