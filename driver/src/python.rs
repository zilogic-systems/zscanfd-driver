use crate::Error;
use crate::{Frame, Interface};
use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use pyo3;
use pyo3::prelude::*;

#[pyclass(name = Interface)]
struct PyInterface {
    i: Interface,
    rx_recvs: Vec<Receiver<Frame>>,
    rx_sends: Vec<Sender<Frame>>,
}

/// Return the Length to DLC.
fn raw_dlc_to_scaled_down_dlc(len : u8) -> u8 {
    match len {
        0..=8   => len,
        9..=12  => 9,
        13..=16 => 10,
        17..=20 => 11,
        21..=24 => 12,
        25..=32 => 13,
        33..=48 => 14,
        49..=64 => 15,
        65..=u8::MAX => 0,
    }
}

/// Return the DLC to length.
#[allow(dead_code)]
fn scaled_down_dlc_to_raw_dlc(dlc : u8) -> u8 {
    match dlc {
        0..=8 => dlc,
        9  => 12,
        10 => 16,
        11 => 20,
        12 => 24,
        13 => 32,
        14 => 48,
        15 => 64,
        16..=u8::MAX => 0,
    }

}

impl IntoPy<PyObject> for Frame {
    fn into_py(self, py: Python) -> PyObject {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("id", self.can_id).unwrap();
        d.set_item("dlc", self.can_dlc).unwrap();
        d.set_item("data", self.data.to_vec()).unwrap();
        d.set_item("extended", self.ext).unwrap();
        d.set_item("rtr", self.rtr).unwrap();
        d.set_item("channel", self.channel).unwrap();
        d.set_item("loopback", self.loopback).unwrap();
        d.set_item("hw_timestamp", self.hw_timestamp).unwrap();
        match self.sw_timestamp {
            Some(t) => d
                .set_item("sw_timestamp", t.as_micros() as f32 / 1000000.0)
                .unwrap(),
            None => d.set_item("sw_timestamp", 0).unwrap(),
        };
        d.to_object(py)
    }
}

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyErr::new::<pyo3::exceptions::SystemError, _>(format!("{:?}", err))
    }
}

#[pymethods]
impl PyInterface {
    #[new]
    #[args(optional_data = "None")]
    fn new(dev_serial_num_opt: Option<String>) -> PyResult<Self> {
        let mut i = match dev_serial_num_opt {
            Some(dev_serial_num) => Interface::new_with_device_serial_number(dev_serial_num)?,
            None => Interface::new()?,
        };

        // disable all channels by default
        for n in 0..i.channels.len() {
            i.set_enabled(n, false)?;
        }

        // Create a N channel of unbounded capacity
        let mut rx_recvs: Vec<Receiver<Frame>> = Vec::new();
        let mut rx_sends: Vec<Sender<Frame>> = Vec::new();

        for _n in 0..i.channels.len() {
            let (send, recv) = unbounded();
            rx_sends.push(send);
            rx_recvs.push(recv);
        }

        Ok(PyInterface {
            i,
            rx_recvs,
            rx_sends,
        })
    }

    fn set_bitrate(&mut self, channel: usize, bitrate: u32) -> PyResult<()> {
        self.i.set_bitrate(channel, bitrate)?;
        Ok(())
    }

    fn set_data_bitrate(&mut self, channel: usize, bitrate: u32) -> PyResult<()> {
        self.i.set_data_bitrate(channel, bitrate)?;
        Ok(())
    }

    fn set_bit_timing(
        &mut self,
        channel: usize,
        brp: u32,
        phase_seg1: u32,
        phase_seg2: u32,
        sjw: u32,
    ) -> PyResult<()> {
        self.i
            .set_bit_timing(channel, brp, phase_seg1, phase_seg2, sjw)?;
        Ok(())
    }


    fn set_data_bit_timing(
        &mut self,
        channel: usize,
        brp: u32,
        phase_seg1: u32,
        phase_seg2: u32,
        sjw: u32,
    ) -> PyResult<()> {
        self.i
            .set_data_bit_timing(channel, brp, phase_seg1, phase_seg2, sjw)?;
        Ok(())
    }

    fn set_enabled(&mut self, channel: usize, enabled: bool) -> PyResult<()> {
        self.i.set_enabled(channel, enabled)?;
        Ok(())
    }

    fn set_monitor(&mut self, channel: usize, enabled: bool) -> PyResult<()> {
        self.i.set_monitor(channel, enabled)?;
        Ok(())
    }

    fn start(&mut self, channel: usize) -> PyResult<()> {
        let rxs = self.rx_sends.clone(); // Clone rx_sends Vector
        self.i.start(channel, move |f: Frame| {
            match rxs[f.channel as usize].send(f) {  // Directly indexing Frame type can never have out of bound channel
                Ok(_) => {}
                Err(_) => {
                    panic!("error starting device")
                }
            };
        })?;

        Ok(())
    }

    fn stop(&mut self, channel: usize) -> PyResult<()> {
        self.i.stop(channel)?;
        Ok(())
    }

    fn recv(&mut self, timeout_ms: u64, channel: usize) -> PyResult<Option<Frame>> {
        if channel > self.i.channels.len() {
            panic!("InvalidChannel");
        }

        let f = match self
            .rx_recvs[channel]
            .recv_timeout(std::time::Duration::from_millis(timeout_ms))
        {
            Ok(f) => f,
            Err(RecvTimeoutError::Timeout) => return Ok(None),
            Err(RecvTimeoutError::Disconnected) => panic!("device thread died"),
        };

        Ok(Some(f))
    }

    fn send(
        &mut self,
        channel: u8,
        id: u32,
        ext: bool,
        rtr: bool,
        dlc: u8,
        data: Vec<u8>,
    ) -> PyResult<()> {
        let mut data_array: Vec<u8> = vec![0; 64];
        for i in 0..dlc as usize {
            if i < dlc.into() {
                data_array[i] = data[i];
            }
        }
        self.i.send(Frame {
            can_id: id,
            can_dlc: dlc,
            ext,
            rtr,
            data: data_array,
            channel,
            loopback: false,
            fd: false,
            brs: false,
            err: false,
            esi: false,
            sw_timestamp: None,
            hw_timestamp: 0,
        })?;
        Ok(())
    }

    fn send_fd(
        &mut self,
        channel: u8,
        id: u32,
        ext: bool,
        rtr: bool,
        fd: bool,
        brs: bool,
        dlc: u8,
        data: Vec<u8>,
    ) -> PyResult<()> {
        let mut data_array: Vec<u8> = vec![0; 64];

        if raw_dlc_to_scaled_down_dlc(dlc) == 0 {
           // Invalid Frame Size Send_fd stop here!
            return Ok(());
        }

        for i in 0..dlc as usize {
            if i < data.len() {
                data_array[i] = data[i];
            }
        }

        self.i.send(Frame {
            can_id: id,
            can_dlc: raw_dlc_to_scaled_down_dlc(dlc),
            ext,
            rtr,
            data: data_array,
            channel,
            loopback: false,
            fd,
            brs,
            err: false,
            esi: false,
            sw_timestamp: None,
            hw_timestamp: 0,
        })?;
        Ok(())
    }

    fn channel_count(&self) -> PyResult<usize> {
        Ok(self.i.channels())
    }

    fn set_fd(&mut self, channel: usize, enabled: bool) -> PyResult<()> {
        self.i.set_fd(channel, enabled)?;
        Ok(())
    }
}

#[pymodule]
fn zscanfd(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyInterface>()?;
    Ok(())
}
