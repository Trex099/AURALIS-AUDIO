use anyhow::Result;
use gstreamer as gst;
use gstreamer::prelude::*;

pub struct BeamSession {
    pipeline: gst::Pipeline,
}

impl BeamSession {
    pub fn new(session_id: &str) -> Result<Self> {
        gst::init()?;
        
        let pipeline_str = format!(
            "webrtcbin name=sendrecv bundle-policy=max-bundle stun-server=stun://stun.l.google.com:19302 \
             audiotestsrc is-live=true wave=red-noise ! opusenc ! rtpopuspay ! sendrecv. \
             "
        );
        
        let pipeline = gst::parse::launch(&pipeline_str)?
            .downcast::<gst::Pipeline>()
            .expect("Expected a pipeline");
            
        // TODO: Connect signals for negotiation
        
        Ok(Self { pipeline })
    }
    
    pub fn start(&self) -> Result<()> {
        self.pipeline.set_state(gst::State::Playing)?;
        Ok(())
    }
    
    pub fn stop(&self) -> Result<()> {
        self.pipeline.set_state(gst::State::Null)?;
        Ok(())
    }
}
