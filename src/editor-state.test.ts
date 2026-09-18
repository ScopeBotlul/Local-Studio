import {describe,it,expect} from 'vitest';
import {appendEdit,undoEdit,redoEdit,emptyHistory} from './editor-state';
describe('editor recipe history',()=>{
 it('branches after undo without retaining the abandoned redo branch',()=>{let h=appendEdit(emptyHistory,{type:'rotate',clockwise:true},100);h=appendEdit(h,{type:'flip',horizontal:true},100);h=undoEdit(h);h=appendEdit(h,{type:'resize',width:10,height:20},100);expect(h.operations.map(o=>o.type)).toEqual(['rotate','resize']);expect(redoEdit(h)).toEqual(h);});
 it('limits undo while retaining operations necessary to render the image',()=>{let h=emptyHistory;for(let i=0;i<4;i++)h=appendEdit(h,{type:'rotate',clockwise:true},2);for(let i=0;i<8;i++)h=undoEdit(h);expect(h.cursor).toBe(2);expect(h.operations).toHaveLength(4);expect(redoEdit(h).cursor).toBe(3);});
});
