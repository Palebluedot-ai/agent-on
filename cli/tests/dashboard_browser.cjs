// Optional browser smoke: node dashboard_browser.cjs URL PLAYWRIGHT_MODULE SCREENSHOT
// Exercises the live read-only view; no dispatch, hooks or Git mutations.
const {chromium}=require(process.argv[3]||'playwright');
(async()=>{
  const browser=await chromium.launch({headless:true,channel:process.env.AGENT_ON_BROWSER_CHANNEL||'chrome'});
  try {
    const page=await browser.newPage({viewport:{width:1360,height:1000}});
    const errors=[];
    page.on('pageerror',error=>errors.push(error.message));
    await page.goto(process.argv[2]);
    await page.waitForFunction(()=>document.getElementById('treeCount').textContent!=='—');
    const state=await page.evaluate(async()=>{
      const response=await fetch('/api/status',{headers:{'X-Agent-On':'dashboard'}});
      return response.json();
    });
    const api=new URL('/api/status',process.argv[2]).href;
    if((await page.request.get(api)).status()!==403)throw Error('API accepts missing header');
    if((await page.request.get(api,{headers:{'X-Agent-On':'dashboard',Origin:'https://evil.test'}})).status()!==403)throw Error('API accepts cross-origin request');
    if((await page.request.post(api)).status()!==405)throw Error('API accepts mutation method');
    if(Number(await page.locator('#treeCount').textContent())!==state.worktrees.length)throw Error('worktree count mismatch');
    if(state.config.data.enabled===false && !(await page.locator('#attention').textContent()).includes('巡逻未启用'))throw Error('missing coverage gap');
    if(state.landing_age_seconds>120 && !(await page.locator('#attention').textContent()).includes('PR 取证'))throw Error('missing stale evidence');
    await page.locator('#filter').fill('this-task-does-not-exist');
    if(await page.locator('#trees td').count())throw Error('filter did not filter');
    await page.locator('#filter').fill('');
    await page.locator('#refresh').click();
    await page.waitForTimeout(200);
    await page.screenshot({path:process.argv[4]||'/tmp/agent-on-dashboard.png',fullPage:true});
    await page.setViewportSize({width:390,height:844});
    if(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1))throw Error('mobile overflow');
    await page.route('**/api/status',route=>route.abort());
    await page.locator('#refresh').click();
    await page.waitForFunction(()=>document.getElementById('connection').textContent.includes('连接已断开'));
    if(Number(await page.locator('#treeCount').textContent())!==state.worktrees.length)throw Error('disconnect lost last observation');
    if(errors.length)throw Error(errors.join('\n'));
    console.log(JSON.stringify({browser:'PASS',worktrees:state.worktrees.length,tasks:state.patrol.data?.tasks?.length??null,console_errors:errors}));
  } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1});
