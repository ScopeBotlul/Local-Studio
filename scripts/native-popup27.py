"""Inspect/select the actual Win32 popup owned by an isolated test process."""
import ctypes as c,json,sys,time
from ctypes import wintypes as w
u=c.WinDLL('user32',use_last_error=True);pid=int(sys.argv[1]);popups=[];owners=[]
u.SetProcessDpiAwarenessContext.argtypes=[c.c_void_p]
u.SetProcessDpiAwarenessContext(c.c_void_p(-4))
u.SendMessageW.argtypes=[w.HWND,w.UINT,w.WPARAM,w.LPARAM];u.SendMessageW.restype=c.c_ssize_t
u.PostMessageW.argtypes=[w.HWND,w.UINT,w.WPARAM,w.LPARAM]
u.GetSubMenu.argtypes=[w.HMENU,c.c_int];u.GetSubMenu.restype=w.HMENU
u.GetMenuItemCount.argtypes=[w.HMENU];u.GetMenuItemCount.restype=c.c_int
u.GetMenuItemID.argtypes=[w.HMENU,c.c_int];u.GetMenuItemID.restype=w.UINT
u.GetMenuState.argtypes=[w.HMENU,w.UINT,w.UINT];u.GetMenuState.restype=w.UINT
u.GetMenuStringW.argtypes=[w.HMENU,w.UINT,w.LPWSTR,c.c_int,w.UINT]
@c.WINFUNCTYPE(w.BOOL,w.HWND,w.LPARAM)
def visit(hwnd,_):
 process=w.DWORD();u.GetWindowThreadProcessId(hwnd,c.byref(process))
 if process.value==pid and u.IsWindowVisible(hwnd):
  name=c.create_unicode_buffer(256);u.GetClassNameW(hwnd,name,256)
  if name.value=='#32768':popups.append(hwnd)
  else:
   rect=w.RECT();u.GetWindowRect(hwnd,c.byref(rect))
   if rect.right-rect.left>500:owners.append(hwnd)
 return True
u.EnumWindows(visit,0)
if not popups or not owners:raise RuntimeError('No native popup found for test process')
def items(menu):
 result=[]
 for i in range(u.GetMenuItemCount(menu)):
  text=c.create_unicode_buffer(1024);u.GetMenuStringW(menu,i,text,1024,0x400);sub=u.GetSubMenu(menu,i)
  result.append(dict(text=text.value.replace('&',''),position=i,id=u.GetMenuItemID(menu,i),enabled=not bool(u.GetMenuState(menu,i,0x400)&3),children=items(sub)if sub else []))
 return result
handle=u.SendMessageW(popups[0],0x1e1,0,0)
result=items(handle)
if len(sys.argv)>2:
 def find(nodes):
  for node in nodes:
   if node['text'].split('\t')[0]==sys.argv[2]:return node
   found=find(node['children'])
   if found:return found
 target=find(result)
 if not target or not target['enabled']:raise RuntimeError('Menu item unavailable: '+sys.argv[2])
 # Muda uses TPM_RETURNCMD. Navigate its tracking loop, without moving the
 # global pointer. Disabled rows also consume one Down key in Win32 menus.
 for _ in range(len(result)+2):
  if u.GetMenuState(handle,target['position'],0x400)&0x80:break
  u.PostMessageW(owners[0],0x0100,0x28,1);time.sleep(.1)
 else:raise RuntimeError('Native menu did not highlight the requested item')
 u.PostMessageW(owners[0],0x0100,0x0d,1)
else:
 u.PostMessageW(owners[0],0x001f,0,0)
 time.sleep(.15)
print(json.dumps(result,ensure_ascii=True))
