// Local browser acceptance. Synthetic responses never enter real project/task ledgers.
// node system_dashboard_browser.cjs URL PLAYWRIGHT_MODULE SCREENSHOT
const {chromium}=require(process.argv[3]||'playwright');
const check=(value,message)=>{if(!value)throw Error(message)};
(async()=>{
  const browser=await chromium.launch({headless:true,channel:process.env.AGENT_ON_BROWSER_CHANNEL||'chrome'});
  try{
    const page=await browser.newPage({viewport:{width:1440,height:1100}}),errors=[];
    page.on('pageerror',error=>errors.push(error.message));
    await page.goto(process.argv[2]);await page.waitForFunction(()=>document.getElementById('projectCount').textContent!=='—');
    const actual=await page.evaluate(async()=>{const response=await fetch('/api/status',{headers:{'X-Agent-On':'dashboard'}});return response.json()});
    check(actual.scope==='system','not system scope');
    check(Number(await page.locator('#projectCount').textContent())===actual.projects.length,'project count mismatch');
    check(await page.locator('#hostFilters [data-host]').count()===6,'missing platform filter');
    check(await page.locator('#tasks [data-lane]').count()===4,'missing task lanes');
    check((await page.locator('#activityStatus').textContent()).includes('未知'),'missing partial activity coverage');
    check(await page.locator('#projects .project').count()===Math.min(actual.projects.length,8),'project summary is not bounded');
    if(actual.projects.length>8){await page.locator('#expandProjects').click();check(await page.locator('#projects .project').count()===actual.projects.length,'all projects not accessible');await page.locator('#expandProjects').click()}
    const api=new URL('/api/status',process.argv[2]).href;
    check((await page.request.get(api)).status()===403,'API missing header accepted');
    check((await page.request.post(api)).status()===405,'API mutation accepted');
    check((await page.request.get(api,{headers:{'X-Agent-On':'dashboard',Origin:'https://evil.test'}})).status()===403,'foreign origin accepted');
    const project=actual.projects.find(p=>p.detail_available);
    if(project){
      const detail=new URL('/api/projects/'+project.id,process.argv[2]).href;
      check((await page.request.get(detail)).status()===403,'detail lacks header guard');
      check((await page.request.get(detail,{headers:{'X-Agent-On':'dashboard'}})).status()===200,'detail unavailable');
      await page.goto(new URL('/projects/'+project.id,process.argv[2]).href);
      await page.waitForFunction(()=>document.getElementById('treeCount').textContent!=='—');
      check((await page.locator('.header-actions a').getAttribute('href'))==='/','detail missing return link');
      await page.goto(process.argv[2]);await page.waitForFunction(()=>document.getElementById('projectCount').textContent!=='—');
    }
    await page.locator('#wall').click();check((await page.locator('body').getAttribute('class')).includes('wall'),'wall mode absent');await page.locator('#wall').click();
    await page.screenshot({path:process.argv[4]||'/tmp/agent-on-system.png',fullPage:true});
    await page.setViewportSize({width:390,height:844});check(!(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1)),'mobile overflow');
    const now=new Date().toISOString(),p1='p1111111111111111',p2='p2222222222222222';
    const fixture={schema_version:2,scope:'system',observed_at:now,roots:['/fixture'],projects:[
      {id:p1,name:'项目一',repo:'/fixture/one',connected:true,available:true,tasks:2,blocked:1,agents:2,hosts:['codex','claude'],worktrees:2,detail_available:true},
      {id:p2,name:'项目二',repo:'/fixture/two',connected:false,available:false,tasks:1,blocked:0,agents:2,hosts:['hermes','antigravity'],worktrees:1,detail_available:true}
    ],tasks:[
      {id:'same',key:p1+':same',project_id:p1,title:'Codex 功能',goal:'具体范围',state:'working',hosts:['codex'],note:'进行中'},
      {id:'tests',key:p1+':tests',project_id:p1,title:'Claude 回归',goal:'验证范围',state:'blocked',hosts:['claude'],note:'账户拒绝'},
      {id:'same',key:p2+':same',project_id:p2,title:'Hermes 功能',goal:'其他范围',state:'reported',hosts:['hermes'],note:'待验收'}
    ],agents:[
      {id:'one',project_id:p1,host:'codex',source:'session-receipt',state:'ready'},
      {id:'two',project_id:p1,host:'claude',source:'self-report',state:'blocked',note:'账户拒绝',stale:true},
      {id:'three',project_id:p2,host:'hermes',source:'process',state:'present',helper:false},
      {id:'four',project_id:p2,host:'antigravity',source:'self-report',state:'working'},
      {id:'five',project_id:null,host:'grok',source:'process',state:'present',helper:true}
    ],errors:[{project_id:p1,project:'项目一',source:'task',id:'tests',note:'账户拒绝'},{project_id:p2,project:'项目二',source:'source',id:'records',note:'记录损坏'}],coverage:'仅 fixture',discovery:{warnings:[]},process_inventory:{available:true}};
    await page.route('**/api/status',route=>route.fulfill({json:fixture}));await page.locator('#refresh').click();
    await page.waitForFunction(()=>document.getElementById('projectCount').textContent==='2');
    check(await page.locator('#tasks .task-card').count()===3,'cross-project tasks missing');
    check((await page.locator('#coverageSummary').textContent()).includes('记录不可读'),'damage hidden');
    check((await page.locator('#hostSignals').textContent()).includes('Hermes'),'Hermes signal missing');
    check((await page.locator('#windowActivity').textContent()).includes('账户拒绝'),'explicit activity not shown');
    check(!(await page.locator('#windowActivity').textContent()).includes('进程存在'),'process became task activity');
    check(!(await page.locator('#hostSignals').textContent()).includes('在线'),'process reported as online');
    await page.locator('#projectFilter').selectOption(p2);check(await page.locator('#tasks .task-card').count()===1,'project filter failed');
    await page.locator('#projectFilter').selectOption('all');await page.locator('[data-host="claude"]').click();
    check(await page.locator('#tasks .task-card').count()===1,'host task filter failed');check((await page.locator('#errors').textContent()).includes('账户拒绝'),'error host filter failed');
    await page.locator('[data-host="all"]').click();
    fixture.tasks[0].state='reported';await page.locator('#refresh').click();
    await page.waitForFunction(()=>!document.querySelector('[data-lane="working"] .task-card'));
    check((await page.locator('[data-lane="working"]').textContent()).includes('窗口活动未知'),'empty ledger mistaken for idle native window');
    const projectActivity = await page.locator('#projects [data-project="'+p1+'"] .offline').textContent();
    check(projectActivity.includes('窗口活动：未知'),'project activity silently shown as zero');
    fixture.tasks[0].state='working';await page.locator('#refresh').click();
    await page.waitForFunction(()=>document.querySelector('[data-lane="working"] .task-card'));
    const details=page.locator(`[data-key="${p1}:tests"] details`);await details.locator('summary').click();
    fixture.tasks[1].state='reported';fixture.errors.shift();await page.locator('#refresh').click();
    await page.waitForFunction(()=>document.querySelector('[data-lane="reported"] [data-key="p1111111111111111:tests"]'));
    check(await details.getAttribute('open')!==null,'expanded task lost on refresh');
    check(await page.locator('[data-lane="blocked"] .task-card').count()===0,'stale task lane');
    fixture.tasks[1].title='<img src=x onerror=window.__xss=1>';fixture.errors[0].note='<svg onload=window.__xss=1>';await page.locator('#refresh').click();
    await page.waitForFunction(()=>document.querySelector('[data-key="p1111111111111111:tests"] h3').textContent.includes('<img'));
    check(await page.locator('#tasks img,#errors svg').count()===0,'HTML executed');check(!(await page.evaluate(()=>window.__xss)),'XSS ran');
    await page.locator('#search').fill('no-such-item');check(await page.locator('#projects .project,#tasks .task-card,#errors .error').count()===0,'search failed');await page.locator('#search').fill('');
    await page.locator('#projectFilter').selectOption(p2);fixture.projects=fixture.projects.filter(p=>p.id!==p2);fixture.tasks=fixture.tasks.filter(t=>t.project_id!==p2);fixture.errors=[];fixture.agents=fixture.agents.filter(a=>a.project_id!==p2);await page.locator('#refresh').click();
    await page.waitForFunction(()=>document.getElementById('projectCount').textContent==='1');check((await page.locator('#projectFilter').inputValue())==='all','removed project did not reset filter');check(await page.locator('#tasks .task-card').count()===2,'removed project left stale filter');
    await page.unroute('**/api/status');await page.locator('#refresh').click();await page.waitForFunction(n=>Number(document.getElementById('projectCount').textContent)===n,actual.projects.length);
    await page.route('**/api/status',route=>route.abort());await page.locator('#refresh').click();await page.waitForFunction(()=>document.getElementById('connection').textContent.includes('连接已断开'));
    check(Number(await page.locator('#projectCount').textContent())===actual.projects.length,'disconnect lost state');
    check(errors.length===0,errors.join('\n'));
    console.log(JSON.stringify({browser:'PASS',projects:actual.projects.length,hosts:actual.hosts,filters:'PASS',dynamic_state:'PASS',detail_route:'PASS',XSS:'PASS',mobile:'PASS',console_errors:errors}));
  }finally{await browser.close()}
})().catch(error=>{console.error(error);process.exitCode=1});
