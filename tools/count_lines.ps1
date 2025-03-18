# Get all .rs files from src and tests directories
$files = @(
    Get-ChildItem -Path "src" -Filter "*.rs" -Recurse
    Get-ChildItem -Path "tests" -Filter "*.rs" -Recurse
)

# Initialize counters
$totalLines = 0
$fileCount = 0

# Sort files by line count
$fileStats = @()
foreach ($file in $files) {
    # Use -Raw to handle different line endings properly
    $lineCount = if (Test-Path $file.FullName) {
        @(Get-Content -Path $file.FullName).Count
    } else {
        0
    }
    $totalLines += $lineCount
    $fileCount++
    # Use Join-Path and platform-agnostic path handling
    $relativePath = $file.FullName.Replace($PWD.Path, "").TrimStart([IO.Path]::DirectorySeparatorChar)
    # Normalize path separators for display
    $relativePath = $relativePath.Replace([IO.Path]::DirectorySeparatorChar, "/")
    $fileStats += [PSCustomObject]@{
        Path = $relativePath
        Lines = $lineCount
    }
}

# Display individual file counts sorted by line count
Write-Host "=== Files by Line Count ==="
$fileStats | Sort-Object -Property Lines -Descending | ForEach-Object {
    Write-Host "$($_.Path): $($_.Lines) lines"
}

# Display summary
Write-Host "`n=== Summary ==="
Write-Host "Total number of .rs files: $fileCount"
Write-Host "Total number of lines: $totalLines"
if ($fileCount -gt 0) {
    Write-Host "Average lines per file: $([math]::Round($totalLines / $fileCount, 2))"
} else {
    Write-Host "No Rust files found."
} 