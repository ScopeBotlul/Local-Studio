"""Isolated installer lifecycle checks; never changes the Windows theme or user app.

Run after npm run desktop:build, using the local Inno Setup compiler.
Test builds use a separate AppId and no shortcuts or app launch action.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import time
import winreg

root = Path(__file__).resolve().parent.parent
artifacts = root / '.artifacts' / f'installer-{int(time.time() * 1000)}'
artifacts.mkdir(parents=True)
report = {'checks': [], 'artifacts': str(artifacts), 'passed': False}
binary = root / 'src-tauri/target/release/local-studio.exe'
expected = hashlib.sha256(binary.read_bytes()).hexdigest()
video_files = json.loads((root / 'src-tauri/video-runtime.json').read_text(encoding='utf-8'))['files']
runtime_files = json.loads((root / 'src-tauri/image-runtime.json').read_text(encoding='utf-8'))['files']
ai_runtime_files = {kind: json.loads((root / f'src-tauri/{kind}-runtime.json').read_text(encoding='utf-8'))['files'] for kind in ['assistant', 'speech']}

def record(text):
    report['checks'].append(text)
    print('PASS', text, flush=True)

def run(executable, arguments, log, destination=None):
    args = [str(executable), '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-', '/LANG=english', f'/LOG={log}']
    if destination:
        assert destination.resolve().is_relative_to(artifacts.resolve())
        args.append(f'/DIR={destination}')
    args += arguments
    result = subprocess.run(args, cwd=root, timeout=90, creationflags=subprocess.CREATE_NO_WINDOW)
    text = log.read_text(encoding='utf-8-sig', errors='replace') if log.exists() else ''
    return result.returncode, text

def compile_test(mode, legacy=False):
    identifier = f'{artifacts.name}-{mode}'
    cmd = ['powershell', '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', str(root / 'scripts/build-installer.ps1'), '-TestBuild', identifier, '-TestTheme', mode if mode != 'legacy' else 'dynamic']
    if legacy:
        cmd.append('-TestLegacy')
    subprocess.run(cmd, cwd=root, check=True, timeout=120)
    return root / 'src-tauri/target/release/bundle/inno' / f'installer-test-{identifier}.exe'

try:
    for mode in ['dynamic', 'light', 'dark']:
        installer = compile_test(mode)
        destination = artifacts / mode
        code, log = run(installer, [], artifacts / f'{mode}-install.log', destination)
        assert code == 0, (code, log[-3000:])
        installed = destination / 'local-studio.exe'
        assert hashlib.sha256(installed.read_bytes()).hexdigest() == expected
        for name, digest in runtime_files.items():
            assert hashlib.sha256((destination / 'image-runtime' / name).read_bytes()).hexdigest() == digest
        for name, digest in video_files.items():
            assert hashlib.sha256((destination / 'video-runtime' / name).read_bytes()).hexdigest() == digest
        for kind, files in ai_runtime_files.items():
            for name, digest in files.items():
                assert hashlib.sha256((destination / f'{kind}-runtime' / name).read_bytes()).hexdigest() == digest
        if mode == 'dynamic':
            try:
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r'Software\Microsoft\Windows\CurrentVersion\Themes\Personalize') as key:
                    use_light = winreg.QueryValueEx(key, 'AppsUseLightTheme')[0]
            except FileNotFoundError:
                use_light = 1
            chosen = 'light' if use_light else 'dark'
            report['windowsAppTheme'] = chosen
        else:
            chosen = mode
        assert f'Local Studio installer theme: {chosen}' in log, log[-3000:]
        identifier = f'{artifacts.name}-{mode}'
        extension = '.localstudio-test-' + identifier
        progid = 'LocalStudio.Test.' + identifier + '.Project'
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, 'Software\\Classes\\' + extension) as key:
            assert winreg.QueryValueEx(key, '')[0] == progid
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, 'Software\\Classes\\' + progid + '\\shell\\open\\command') as key:
            assert winreg.QueryValueEx(key, '')[0] == f'"{installed}" "%1"'
        record(f'{mode}: isolated project association and quoted open command installed')
        record(f'{mode}: actual installer selects {chosen}; installed executable and all four runtimes match release')

        keep = destination / 'user-kept-file.txt'
        keep.write_text('preserve user files', encoding='utf-8')
        code, log = run(installer, [], artifacts / f'{mode}-upgrade.log', destination)
        assert code == 0 and keep.read_text(encoding='utf-8') == 'preserve user files'
        assert hashlib.sha256(installed.read_bytes()).hexdigest() == expected
        record(f'{mode}: reinstall preserves added user files')

        code, log = run(destination / 'unins000.exe', [], artifacts / f'{mode}-uninstall.log')
        assert code == 0, (code, log[-3000:])
        assert f'Local Studio uninstaller theme: {chosen}' in log, log[-3000:]
        assert not installed.exists() and keep.read_text(encoding='utf-8') == 'preserve user files'
        assert all(not (destination / 'image-runtime' / name).exists() for name in runtime_files)
        assert all(not (destination / 'video-runtime' / name).exists() for name in video_files)
        assert all(not (destination / f'{kind}-runtime' / name).exists() for kind, files in ai_runtime_files.items() for name in files)
        for key_path in ['Software\\Classes\\' + progid, 'Software\\Classes\\' + extension]:
            try:
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path):
                    raise AssertionError('Association remained after uninstall: ' + key_path)
            except FileNotFoundError:
                pass
        record(f'{mode}: isolated file association removed on uninstall')
        record(f'{mode}: uninstall selects {chosen}, removes program and preserves added files')

        if mode == 'dynamic':
            portable = artifacts / 'portable-protected'
            portable.mkdir()
            (portable / 'portable.marker').write_text('test-only', encoding='utf-8')
            code, log = run(installer, [], artifacts / 'portable-block.log', portable)
            assert code != 0 and not (portable / 'local-studio.exe').exists()
            assert 'portable version' in log
            record('Portable destination rejected before copying application files')

    legacy = compile_test('legacy', legacy=True)
    target = artifacts / 'legacy-protected'
    code, log = run(legacy, [], artifacts / 'legacy-block.log', target)
    assert code != 0 and not (target / 'local-studio.exe').exists()
    assert 'legacy NSIS installation blocked' in log
    record('Simulated legacy NSIS detection blocks installation without modifying existing files')
    report['passed'] = True
finally:
    (artifacts / 'report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print('Report:', artifacts / 'report.json', flush=True)
