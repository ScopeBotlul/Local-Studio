import {describe,expect,it} from 'vitest';
import {notificationButtonLabel} from './NotificationCenter';

describe('notification center labels',()=>{
  it('uses correct singular and plural labels',()=>{
    expect(notificationButtonLabel(0,true)).toBe('Keine Benachrichtigungen');
    expect(notificationButtonLabel(1,true)).toBe('1 Benachrichtigung');
    expect(notificationButtonLabel(2,false)).toBe('2 notifications');
  });
});
