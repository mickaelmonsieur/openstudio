# Streaming Encoder

OpenStudio can stream to Icecast in MP3 with LAME. AAC-LC support is optional and uses Fraunhofer FDK AAC through dynamic loading.

OpenStudio does not bundle `libfdk-aac` because of its license and patent situation. If the library is not installed, the AAC-LC option remains visible but disabled in the Streaming Encoder window.

## macOS

Install FDK AAC with Homebrew:

```bash
brew install fdk-aac
```

OpenStudio looks for:

- `/opt/homebrew/lib/libfdk-aac.dylib`
- `/opt/homebrew/lib/libfdk-aac.2.dylib`
- `/usr/local/lib/libfdk-aac.dylib`
- `/usr/local/lib/libfdk-aac.2.dylib`
- `/Library/Application Support/butt/libfdk-aac.2.dylib`

On Apple Silicon, Homebrew usually installs the library under `/opt/homebrew/lib`.

You can verify the installation with:

```bash
ls -l /opt/homebrew/lib/libfdk-aac*.dylib
```

## Linux

Install the distribution package that provides `libfdk-aac.so` or `libfdk-aac.so.2`.

On Debian or Ubuntu, this is typically:

```bash
sudo apt update
sudo apt install libfdk-aac2
```

Depending on the distribution, the package may be in a non-free, restricted, or multiverse repository.

OpenStudio looks for:

- `libfdk-aac.so`
- `libfdk-aac.so.2`
- `/usr/lib/libfdk-aac.so`
- `/usr/local/lib/libfdk-aac.so`
- `/usr/lib/x86_64-linux-gnu/libfdk-aac.so`
- `/lib/x86_64-linux-gnu/libfdk-aac.so`

You can verify the installation with:

```bash
ldconfig -p | grep fdk-aac
```

## Windows

Install FDK AAC with MSYS2:

1. Install MSYS2 from <https://www.msys2.org/>.
2. Open the **UCRT64** MSYS2 shell.
3. Install FDK AAC:

```bash
pacman -S mingw-w64-ucrt-x86_64-fdk-aac
```

OpenStudio looks for these DLL names through the normal Windows DLL search path:

- `libfdk-aac-2.dll`
- `fdk-aac.dll`
- `libfdk-aac.dll`

Recommended setup:

1. Locate `libfdk-aac-2.dll`, usually in `C:\msys64\ucrt64\bin`.
2. Add `C:\msys64\ucrt64\bin` to the system `PATH`, or copy `libfdk-aac-2.dll` next to `OpenStudio.exe`.

If you copy the DLL manually and OpenStudio still does not enable AAC-LC, the FDK DLL may require additional runtime DLLs from the same MSYS2 `bin` directory. In that case, keeping `C:\msys64\ucrt64\bin` in `PATH` is the simplest option.

## References

- FDK AAC shared-library project: <https://github.com/mstorsjo/fdk-aac>
- MSYS2: <https://www.msys2.org/>
