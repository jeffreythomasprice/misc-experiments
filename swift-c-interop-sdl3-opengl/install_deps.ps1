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

New-Item -ItemType Directory -Force -Path GL
DownloadHelper -URL "https://www.opengl.org/registry/api/GL/glext.h" -OutFile "GL/glext.h"
DownloadHelper -URL "https://www.opengl.org/registry/api/GL/glcorearb.h" -OutFile "GL/glcorearb.h"
DownloadHelper -URL "https://www.opengl.org/registry/api/GL/glxext.h" -OutFile "GL/glxext.h"
DownloadHelper -URL "https://www.opengl.org/registry/api/GL/wglext.h" -OutFile "GL/wglext.h"
Copy-Item -Path "GL" -Destination "../Sources/COpenGL/" -Recurse -Force

cd ..