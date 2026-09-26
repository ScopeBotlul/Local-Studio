import assert from 'node:assert/strict';

const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));

export async function checkTags37({invoke,record}) {
  const owner=crypto.randomUUID();
  const postUrl='https://rule34.xxx/index.php?page=post&s=view&id=1887067';
  await invoke('civitai_browser_mount',{owner,bounds:{x:24,y:120,width:900,height:620}});
  try {
    await invoke('civitai_browser_action',{owner,action:{kind:'visit',url:postUrl}});
    let state;
    for(let attempt=0;attempt<120;attempt++){
      state=await invoke('civitai_browser_state');
      if(state.postId===1887067&&!state.loading)break;
      await pause(250);
    }
    assert.equal(state?.postId,1887067,'Rule34 post URL was not recognized by the integrated browser');
    const tags=await invoke('tag_post_tags',{url:state.url});
    assert.match(tags,/^(?:Character|General|Copyright|Artist|Metadata)\n/m);
    assert.ok(tags.split('\n').filter(Boolean).length>3,'Expected tags extracted from the loaded post');
    record('Rule34 post tags load from the integrated browser through typed IPC',`${tags.split('\n').filter(Boolean).length-1} entries`);
  } finally {
    await invoke('civitai_browser_hide',{owner}).catch(()=>{});
  }
}
