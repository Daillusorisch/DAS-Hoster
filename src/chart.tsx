import React, { useState } from 'react';
import Plot from "react-plotly.js"
import { PlotParams } from 'react-plotly.js';
import { Channel } from '@tauri-apps/api/core';
import Plotly from 'plotly.js-dist-min';

// class ChartView extends React.Component {
//     on_data: Channel<Plotly.Data>;
//     state: Readonly<PlotParams>;
//     constructor(channel: Channel<Plotly.Data>) {
//         super(channel);
        // let data = [{}];
        // for(let i =0; i<7;i++){
        //     data.push({
        //         x:figure.data[i].x, 
        //         y:figure.data[i].y,
        //         z:figure.data[i].z,
        //         name: '',    
        //         colorscale: figure.data[i].colorscale,
        //         type: 'surface',
        //         showscale: false
        //     });
//         // }


type Props = {
    channel: Channel<Plotly.Data>;
}

const rootTag = "graph";
const type = "surface";
const showscale = false;
const name = "init";
const initState: PlotParams = {
    data:[{x:[[0,0]],y:[[0,0]],z:[[0,0]],type,showscale,name}],
    layout:{
        title:"DAS",
        showlegend: false,
        autosize: true,
        width: 800,
        height: 700,
        scene: {
            xaxis: {title: 'distince (m)'},
            yaxis: {title: 'time (s)'},
            zaxis: {title: 'strain'}
      }
    },
    config: {},
    frames:[]
};

export const ChartView: React.FC<Props> = (Props) => {
    const on_data = Props.channel;
    // const [data, setData] = useState(initState.data);
    // console.log(data);
    // const [layout, setLayout] = useState(initState.layout);
    const [state, setState] = useState(initState);
    let data = state.data;
    on_data.onmessage = (msg) => {
        if (data.length === 1 && data[0].name === "init") {
            data.pop()
        }
        data.push(msg);
        setState({...state, data});
        // console.log(state.data);
    }

    function clearChart() {
        data = [{x:[[0,0]],y:[[0,0]],z:[[0,0]],type,showscale,name}];
        // console.log(data);
        setState({...state, data})
    }

    return (
        <div>
            <button onClick={clearChart}>重置图像</button>
            <div id={rootTag}>
                <Plot
                    data={state.data}
                    layout={state.layout}
                    frames={[]}
                    config={{}}
                    // onInitialized={(figure) => setState(figure)}
                    // onUpdate={(figure) => setState(figure)}
                />
            </div>
        </div>
    );
}


// export default ChartView;