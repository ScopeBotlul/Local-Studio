"""Exercise the actual Win32 menu of a specified isolated test process."""
import ctypes as c,json,sys
from ctypes import wintypes as w
u=c.WinDLL('user32',use_last_error=True)
u.GetMenu.argtypes=[w.HWND];u.GetMenu.restype=w.HMENU
u.GetSubMenu.argtypes=[w.HMENU,c.c_int];u.GetSubMenu.restype=w.HMENU
u.GetMenuItemCount.argtypes=[w.HMENU];u.GetMenuItemCount.restype=c.c_int
u.GetMenuItemID.argtypes=[w.HMENU,c.c_int];u.GetMenuItemID.restype=w.UINT
u.GetMenuState.argtypes=[w.HMENU,w.UINT,w.UINT];u.GetMenuState.restype=w.UINT
u.GetMenuStringW.argtypes=[w.HMENU,w.UINT,w.LPWSTR,c.c_int,w.UINT]
u.GetWindowThreadProcessId.argtypes=[w.HWND,c.POINTER(w.DWORD)]
u.PostMessageW.argtypes=[w.HWND,w.UINT,w.WPARAM,w.LPARAM]
pid=int(sys.argv[1]);windows=[]
@c.WINFUNCTYPE(w.BOOL,w.HWND,w.LPARAM)
def visit(hwnd,_):
    process=w.DWORD();u.GetWindowThreadProcessId(hwnd,c.byref(process))
    if process.value==pid and u.GetMenu(hwnd): windows.append(hwnd)
    return True
u.EnumWindows(visit,0)
if not windows: raise RuntimeError('No native menu found for test process')
hwnd=windows[0]
def items(menu):
    result=[]
    for i in range(u.GetMenuItemCount(menu)):
        text=c.create_unicode_buffer(1024);u.GetMenuStringW(menu,i,text,1024,0x400)
        sub=u.GetSubMenu(menu,i)
        result.append(dict(text=text.value.replace('&',''),id=u.GetMenuItemID(menu,i),enabled=not bool(u.GetMenuState(menu,i,0x400)&3),children=items(sub) if sub else []))
    return result
result=items(u.GetMenu(hwnd))
if len(sys.argv)>2:
    def find(nodes):
        for node in nodes:
            if node['text'].split('\t')[0]==sys.argv[2]:return node
            found=find(node['children'])
            if found:return found
    target=find(result)
    if not target or not target['enabled']:raise RuntimeError('Menu item unavailable: '+sys.argv[2])
    if not u.PostMessageW(hwnd,0x111,target['id'],0):raise c.WinError(c.get_last_error())
print(json.dumps(result,ensure_ascii=True))
