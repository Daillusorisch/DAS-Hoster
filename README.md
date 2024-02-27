# das hoster

## data pipeline

```bash
usbport or serialport 
    --read-> buffer: [u8] 
    --parse-> DASFrameRaw 
    --to_das_frame-> DASFrame
    --plotly-> draw
```
