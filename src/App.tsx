import { useState } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import {ChartView} from './chart';
import "./App.css";

function App() {
  const [speed, setSpeed] = useState(0);
  const [length, setLength] = useState(0);
  const [click, setClick] = useState(false);
  const [rid, setRid] = useState(0);
  const [state, setState] = useState("未连接");
  const [dasRid, setDasRid] = useState(0);
  
  const channel = new Channel<Plotly.Data>();
  channel.onmessage = (data) => {
    console.log("2",data);
  }

  async function recive_data():Promise<number> {
    let id = await invoke<number>("start", { 
        onData: channel, info:{
          length: length || 1024.0,
          sample_rate: speed || 96000000,
          pulse_interval: 100000
        }
    });
    console.log("start",id);
    return id;
  }

  async function stop():Promise<number> {
    console.log("stop", rid);
    return await invoke("stop",{rid});
  }

  async function connect_das():Promise<number> {
    console.log("connect_das");
    setState("连接中...");
    return await invoke("connect_das",{usbId:65036});
  }

  async function disconnect_das():Promise<number> {
    console.log("disconnect_das");
    return await invoke("disconnect_das",{dasRid});
  }

  return (
    <div className="container">
      <div>{state}</div>
        <div>
          <label>等效采样速度
            <input type="number" value={speed} required onChange={(e) => setSpeed(parseInt(e.target.value))} />
          </label>
        </div>
        <div>
          <label>光纤长度
            <input type="number" value={length} required onChange={(e) => setLength(parseFloat(e.target.value))} />
          </label>
        </div>
        <div>
          <button onClick={async () => await connect_das().then((das_rid)=>{setDasRid(das_rid);setState("已连接")}).catch(()=>setState("连接失败"))} >连接设备</button>
          <button onClick={async () => await disconnect_das().then((res)=>{setDasRid(res);setState("未连接")})} >断开设备</button>
          <button onClick={async ()=>{if (state === "未连接" || state === "连接失败") {alert("设备未连接");return;}setRid(click?await stop():await recive_data());setClick(!click);}}>{click ? '停止' : '开始'}</button>
        </div>
      <ChartView channel={channel}/>
    </div>
  );
}

export default App;
