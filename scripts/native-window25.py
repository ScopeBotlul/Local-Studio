import ctypes, sys
from ctypes import wintypes
user = ctypes.WinDLL('user32', use_last_error=True)
user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
user.IsWindowVisible.argtypes = [wintypes.HWND]
user.GetMenu.argtypes = [wintypes.HWND]
user.GetMenu.restype = wintypes.HMENU
user.ShowWindow.argtypes = [wintypes.HWND, ctypes.c_int]
windows=[]
@ctypes.WINFUNCTYPE(wintypes.BOOL,wintypes.HWND,wintypes.LPARAM)
def visit(hwnd,_):
    owner=wintypes.DWORD()
    user.GetWindowThreadProcessId(hwnd,ctypes.byref(owner))
    if owner.value==int(sys.argv[1]):
        name=ctypes.create_unicode_buffer(256)
        user.GetClassNameW(hwnd,name,256)
        title=ctypes.create_unicode_buffer(1024);user.GetWindowTextW(hwnd,title,1024)
        rect=wintypes.RECT();user.GetWindowRect(hwnd,ctypes.byref(rect))
        if name.value!='#32768' and 'Local Studio' in title.value: windows.append(hwnd)
    return True
user.EnumWindows(visit,0)
if len(windows)!=1: raise RuntimeError(f'Expected one main window, found {len(windows)}')
if sys.argv[2]=='visible': print('true' if user.IsWindowVisible(windows[0]) else 'false')
elif sys.argv[2]=='show': user.ShowWindow(windows[0],9)
