// SPIKE ONLY. Compiled by the Windows built-in PowerShell Add-Type facility.
// Uses OS/.NET APIs only; no SDK, Rust, Python, certificate changes or sidecars.
using System;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
public static class AfterglowSpikeCNative {
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
    struct SHFILEINFO {
        public IntPtr hIcon;
        public int iIcon;
        public uint dwAttributes;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst=260)] public string szDisplayName;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst=80)] public string szTypeName;
    }
    [DllImport("shell32.dll", CharSet=CharSet.Unicode, EntryPoint="SHGetFileInfoW")]
    static extern IntPtr SHGetFileInfo(string path, uint attributes, out SHFILEINFO info, uint size, uint flags);
    [DllImport("user32.dll")] static extern bool DestroyIcon(IntPtr icon);
    public static void SaveShellIcon(string exe, string png) {
        SHFILEINFO info;
        // Actual file path, SHGFI_ICON. No SHGFI_USEFILEATTRIBUTES generic icon.
        if (SHGetFileInfo(exe, 0, out info, (uint)Marshal.SizeOf(typeof(SHFILEINFO)), 0x100) == IntPtr.Zero || info.hIcon == IntPtr.Zero)
            throw new IOException("Windows Shell could not extract the actual executable icon.");
        try {
            using (Icon icon=Icon.FromHandle(info.hIcon))
            using (Bitmap bitmap=icon.ToBitmap()) bitmap.Save(png, ImageFormat.Png);
        } finally { DestroyIcon(info.hIcon); }
    }
    static async Task<byte[]> ReadBounded(Stream stream) {
        using (MemoryStream bytes=new MemoryStream()) {
            byte[] buffer=new byte[1024];
            int count;
            while ((count=await stream.ReadAsync(buffer,0,buffer.Length).ConfigureAwait(false))!=0) {
                if (bytes.Length+count>16384) throw new IOException("Resource output exceeds 16 KiB limit.");
                bytes.Write(buffer,0,count);
            }
            return bytes.ToArray();
        }
    }
    public static byte[] ReadResource(string exe, string folder, string kind) {
        ProcessStartInfo start=new ProcessStartInfo(exe,kind);
        start.WorkingDirectory=folder;
        start.UseShellExecute=false;
        start.CreateNoWindow=true;
        start.RedirectStandardOutput=true;
        start.RedirectStandardError=true;
        using (Process process=Process.Start(start)) {
            Task<byte[]> output=ReadBounded(process.StandardOutput.BaseStream);
            Task<byte[]> errors=ReadBounded(process.StandardError.BaseStream);
            try {
                // A child with excessive output must not be allowed to hang the collector.
                Stopwatch elapsed=Stopwatch.StartNew();
                while (!process.WaitForExit(50)) {
                    if (output.IsFaulted || errors.IsFaulted) throw new IOException("Resource output limit exceeded.");
                    if (elapsed.Elapsed.TotalSeconds>=30) throw new TimeoutException("Standalone resource reader timed out.");
                }
                if (!Task.WaitAll(new Task[]{output,errors},5000)) throw new TimeoutException("Resource streams did not finish.");
                if (process.ExitCode!=0) throw new IOException("Standalone resource reader failed: "+System.Text.Encoding.UTF8.GetString(errors.Result));
                if (output.Result.Length==0) throw new IOException("Resource was empty.");
                return output.Result;
            } finally { if (!process.HasExited) process.Kill(); }
        }
    }
}
