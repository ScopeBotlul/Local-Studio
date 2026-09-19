import {afterEach,describe,expect,it,vi} from 'vitest';
import {scheduleStartupUpdate} from './startup-update';
import type {UpdateStatus} from './update-api';

const state=(phase:UpdateStatus['phase']):UpdateStatus=>({phase,portable:false,latest:phase==='available'?{version:'99.0.0',notes:'Update',publishedAt:'',installer:{url:'',size:1,sha256:''},portable:{url:'',size:1,sha256:''}}:null,received:0,total:0,error:null});
function setup(phase:UpdateStatus['phase']='available'){
 vi.useFakeTimers();
 const options={status:vi.fn().mockResolvedValue(state('idle')),check:vi.fn().mockResolvedValue(state(phase)),canShow:vi.fn(()=>true),onStarted:vi.fn(),onAvailable:vi.fn()};
 const stop=scheduleStartupUpdate(options);
 return {...options,stop};
}
afterEach(()=>vi.useRealTimers());
describe('automatic startup update notification',()=>{
 it('checks once after startup and offers a new release without downloading',async()=>{
  const s=setup();await vi.advanceTimersByTimeAsync(14999);expect(s.check).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(1);expect(s.check).toHaveBeenCalledTimes(1);expect(s.onAvailable).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(60000);expect(s.check).toHaveBeenCalledTimes(1);expect(s.onAvailable).toHaveBeenCalledTimes(1);s.stop();
 });
 it.each(['current','error','downloading'] as const)('does not notify for %s',async phase=>{
  const s=setup(phase);await vi.advanceTimersByTimeAsync(15000);expect(s.onAvailable).not.toHaveBeenCalled();s.stop();
 });
 it('waits for another dialog to close',async()=>{
  const s=setup();s.canShow.mockReturnValue(false);await vi.advanceTimersByTimeAsync(16000);expect(s.onAvailable).not.toHaveBeenCalled();
  s.canShow.mockReturnValue(true);await vi.advanceTimersByTimeAsync(500);expect(s.onAvailable).toHaveBeenCalledTimes(1);s.stop();
 });
 it('does not check after the setting is disabled before the timer',async()=>{
  const s=setup();s.stop();await vi.advanceTimersByTimeAsync(20000);expect(s.check).not.toHaveBeenCalled();
 });
 it('ignores an in-flight response after disabling or unmounting',async()=>{
  const s=setup();let finish!:(status:UpdateStatus)=>void;s.check.mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));
  await vi.advanceTimersByTimeAsync(15000);s.stop();finish(state('available'));await vi.advanceTimersByTimeAsync(1000);expect(s.onAvailable).not.toHaveBeenCalled();
 });
 it('keeps network failures quiet at startup',async()=>{
  const s=setup();s.check.mockRejectedValue('update_network');await vi.advanceTimersByTimeAsync(15000);expect(s.onAvailable).not.toHaveBeenCalled();s.stop();
 });
});
