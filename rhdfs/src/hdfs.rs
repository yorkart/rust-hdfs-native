
use cmdx::*;
use nn::*;
use dt::*;
use protobuf_api::*;
use proto_tools::*;
#[allow(unused_imports)]
use futures::prelude::*;
use *;

pub use cmdx::Mdx;


//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct GetListingState {
    source: String,
    need_location: bool
}

impl GetListingState {
    fn new(source: String, need_location: bool) -> GetListingState {
        GetListingState { source, need_location }
    }

    #[inline]
    fn q(&self, start: Vec<u8>) -> GetListingRequestProto {
        pb_cons!(GetListingRequestProto,
                    src: self.source.clone(),
                    start_after: start,
                    need_location: self.need_location
                    )
    }

    fn s(&mut self) -> GetListingRequestProto {
        self.q(vec![])
    }

    fn n(&mut self, nr: GetListingResponseProto) -> (Vec<HdfsFileStatusProto>, Option<GetListingRequestProto>) {
        let dir_list = pb_decons!(GetListingResponseProto, nr, dir_list);
        let (fs, remaining_entries) = pb_decons!(DirectoryListingProto, dir_list,
                    partial_listing, remaining_entries);

        if remaining_entries == 0 {
            (fs, None)
        } else {
            let last_filename = Vec::from(
                fs.last().map(|o| pb_get!(HdfsFileStatusProto, o, path)).unwrap_or(&[])
            );
            (fs, Some(self.q(last_filename)))
        }
    }
}

impl ChatF for GetListingState {
    type NQ = MdxQ;
    type NR = MdxR;
    type UR = Vec<HdfsFileStatusProto>;

    fn start(&mut self) -> MdxQ {
        MdxQ::NN(0, None, NnaQ::new(NnQ::GetListing(self.s())))
    }

    fn next(&mut self, nr: MdxR) -> Result<(Vec<HdfsFileStatusProto>, Option<MdxQ>)> {
        match nr {
            MdxR::NN(_, NnaR { inner: NnR::GetListing(glrp)}) => {
                let (a, ob) = self.n(glrp);
                Ok((a, ob.map(|n| MdxQ::NN(0, None, NnaQ::new(NnQ::GetListing(n))))))
            }
            other =>
                Err(app_error!(other "Unexpected {:?} where NN(GetListing) is expected", other))
        }
    }
}


pub type GetListingStream = chat::T<Mdx, GetListingState>;

pub fn get_listing(mdx: Mdx, source: String, need_location: bool) -> GetListingStream {
    chat::new(mdx, GetListingState::new(source, need_location))
}


#[test]
fn test_get_listing_() {
    init_env_logger!();

    use util::test::ptk::*;
    let host_port = "127.0.0.1:58119";
    let t = spawn_test_server(host_port, test_script! {

    expect "68:72:70:63:09:00:00:00:00:00:1e:10:08:02:10:00:18:05:22:08:01:02:03:04:04:03:02:01:\
    0c:12:0a:0a:08:63:6c:6f:75:64:65:72:61:00:00:00:58:10:08:02:10:00:18:00:22:08:01:02:03:04:04:\
    03:02:01:3e:0a:0a:67:65:74:4c:69:73:74:69:6e:67:12:2e:6f:72:67:2e:61:70:61:63:68:65:2e:68:61:\
    64:6f:6f:70:2e:68:64:66:73:2e:70:72:6f:74:6f:63:6f:6c:2e:43:6c:69:65:6e:74:50:72:6f:74:6f:63:\
    6f:6c:18:01:07:0a:01:2f:12:00:18:00",

    send "00:00:01:70:12:08:00:10:00:18:09:3a:08:01:02:03:04:04:03:02:01:40:01:db:02:0a:d8:02:0a:\
    3d:08:01:12:0a:62:65:6e:63:68:6d:61:72:6b:73:18:00:22:03:08:ff:03:2a:04:68:64:66:73:32:0a:73:\
    75:70:65:72:67:72:6f:75:70:38:e1:d7:df:d6:d5:2b:40:00:50:00:58:00:68:8e:80:01:70:00:80:01:00:\
    0a:39:08:01:12:05:68:62:61:73:65:18:00:22:03:08:ed:03:2a:05:68:62:61:73:65:32:0a:73:75:70:65:\
    72:67:72:6f:75:70:38:b4:be:e4:95:f7:2b:40:00:50:00:58:00:68:8d:80:01:70:09:80:01:00:0a:31:08:\
    01:12:04:73:6f:6c:72:18:00:22:03:08:ed:03:2a:04:73:6f:6c:72:32:04:73:6f:6c:72:38:e1:91:e8:d6:\
    d5:2b:40:00:50:00:58:00:68:f9:81:01:70:00:80:01:00:0a:36:08:01:12:03:74:6d:70:18:00:22:03:08:\
    ff:07:2a:04:68:64:66:73:32:0a:73:75:70:65:72:67:72:6f:75:70:38:eb:b2:ab:b4:97:2c:40:00:50:00:\
    58:00:68:84:80:01:70:05:80:01:00:0a:37:08:01:12:04:75:73:65:72:18:00:22:03:08:ed:03:2a:04:68:\
    64:66:73:32:0a:73:75:70:65:72:67:72:6f:75:70:38:b7:b5:e6:d6:d5:2b:40:00:50:00:58:00:68:82:80:\
    01:70:08:80:01:00:0a:36:08:01:12:03:76:61:72:18:00:22:03:08:ed:03:2a:04:68:64:66:73:32:0a:73:\
    75:70:65:72:67:72:6f:75:70:38:a4:f2:e5:d6:d5:2b:40:00:50:00:58:00:68:85:80:01:70:02:80:01:00:\
    10:00"
    });

    use std::net::ToSocketAddrs;

    let addr = host_port.to_socket_addrs().unwrap().next().unwrap();

    let session_data = SessionData {
        effective_user: "cloudera".to_owned(),
        forced_client_id: Some(vec![1, 2, 3, 4, 4, 3, 2, 1])
    };
    let mdx = Mdx::new(1, 1, session_data,  addr, vec![]);

    let gls = get_listing(mdx, "/".to_owned(), false);

    let result: Vec<HdfsFileStatusProto> =
        gls.map(|s| futures::stream::iter_ok::<_, Error>(s.into_iter()))
            .flatten()
            .collect()
            .wait()
            .expect("gls.wait()");

    let y: Vec<Cow<str>> = result.iter().map(|fs| String::from_utf8_lossy(fs.get_path())).collect();
    let z: Vec<Cow<str>> =(["benchmarks", "hbase", "solr", "tmp", "user", "var"]).iter().map(|x|Cow::from(*x)).collect();
    assert_eq!(y, z);

    //-----------------------------------
    let _ = t.join().unwrap();

}

//--------------------------------------------------------------------------------------------------
// Get command
//--------------------------------------------------------------------------------------------------

use bytes::Bytes;
use std::collections::VecDeque;
use dt::ReadBlock;

enum GetFsmEvent {
    Init,
    /// Acknowledge reception of the packet. Arg is `data_len`.
    PacketAck(u64),
    /// Signal that there was an error, probably receiving the packet (CRC or whatever else)
    Err(Error),
    /// End of stream has been received when reading a block
    BlockComplete,
    /// Block locations have been received
    BlockLocations(Vec<LocatedBlock>)
}

enum GetFsmAction {
    NOP,
    RequestBlockLocations { offset: u64, len: u64 },
    ReadBlock(DatanodeInfo, ReadBlock),
    Err(Error),
    Success
}

/*
#[derive(Debug)]
pub struct DatanodeID {
    datanode_uuid: String,
    ip_addr: String,
    xfer_port: u32
}

#[derive(Debug)]
pub struct DatanodeInfo {
    id: DatanodeID
}
*/
#[derive(Debug, Clone)]
pub struct DatanodeInfo {
    inner: DatanodeInfoProto
}

impl DatanodeInfo {
    pub fn into_xfer_info(self) -> (String, String, u32) {
        let id = pb_decons!(DatanodeInfoProto, self.inner, id);
        pb_decons!(DatanodeIDProto, id, datanode_uuid, ip_addr, xfer_port)
    }
}

impl Into<DatanodeInfoProto> for DatanodeInfo {
    fn into(self) -> DatanodeInfoProto {
        self.inner
    }
}

impl From<DatanodeInfoProto> for DatanodeInfo {
    fn from(inner: DatanodeInfoProto) -> Self {
        DatanodeInfo { inner }
    }
}


#[derive(Debug)]
struct LocatedBlock {
    /// Offset of the block within the file
    o: u64,
    b: ExtendedBlock,
    t: Token,
    /// remaining datanodes
    locs: VecDeque<DatanodeInfo>
}

impl LocatedBlock {
    /// Builds ReadBlock action given `[read_offset, max_read_offset)`.
    /// Returns `GetFsmAction::ReadBlock` on success, `GetFsmAction::NOP` if `read_offset` is
    /// ahead of the block's range, various `GetFsmAction::Err` otherwise.
    fn build_read_block(&mut self, read_offset: u64, max_read_offset: u64) -> GetFsmAction {
        let lb = self.o;
        let ub = lb + self.b.get_num_bytes();
        let lb_ok = lb <= read_offset;
        let ub_ok = read_offset < ub;
        if lb_ok && ub_ok {
            //fetch next DNI
            if let Some(dni)  = self.locs.pop_front() {
                GetFsmAction::ReadBlock(dni, ReadBlock {
                    b: self.b.clone(),
                    t: self.t.clone(),
                    offset: read_offset - lb,
                    len: max_read_offset.min(ub) - read_offset
                })
            } else {
                //no valid replica
                GetFsmAction::Err(app_error!(other "All replicas of block {:?} are corrupt", self.b))
            }
        } else if !ub_ok {
            //Skip to the next block
            GetFsmAction::NOP
        } else {
            //error -- gap between blocks. NN corruption.
            GetFsmAction::Err(app_error!(other
            "Current block in chain falls behind the read pointer (gap between blocks) on {:?}: \
            current block's range: [{}, {}), read range [{}, {})",
                self.b, lb, ub, read_offset, max_read_offset
            ))
        }
    }
}

struct GetFsm {
    q: VecDeque<LocatedBlock>,
    /// Upper bound of `read_offset` (size of the file)
    max_read_offset: u64,
    /// file position read successfully so far. This is at a packet boundary or at the end.
    read_offset: u64
}

impl GetFsm {
    fn new(read_offset: u64, max_read_offset: u64) -> GetFsm {
        GetFsm { q: VecDeque::new(), read_offset, max_read_offset }
    }

    fn adjust_max_read_offset(&mut self, max_read_offset: u64) {
        self.max_read_offset = max_read_offset.min(self.max_read_offset);
    }

    fn next_block(&mut self) -> GetFsmAction {
        if self.read_offset < self.max_read_offset {
            loop {
                let a = match self.q.front_mut() {
                    Some(located_block) =>
                        located_block.build_read_block(self.read_offset, self.max_read_offset),
                    None =>
                        GetFsmAction::RequestBlockLocations {
                            offset: self.read_offset,
                            len: self.max_read_offset - self.read_offset
                        }
                };
                if let GetFsmAction::NOP = a {
                    //skip the block and start over
                    self.q.pop_front();
                } else {
                    break a
                }
            }
        } else {
            GetFsmAction::Success
        }
    }

    fn handle(&mut self, evt: GetFsmEvent) -> GetFsmAction {
        use self::GetFsmEvent as E;
        use self::GetFsmAction as A;

        match evt {
            E::Init | E::BlockComplete =>
                self.next_block(),
            E::Err(e) => if self.q.is_empty() {
                A::Err(e)
            } else {
                self.next_block()
            }
            E::BlockLocations(blocks) =>
                if blocks.is_empty() {
                    A::Err(app_error!(other "invalid empty BlockLocations"))
                } else {
                    self.q.extend(blocks.into_iter());
                    self.next_block()
                }
            E::PacketAck(len) => {
                self.read_offset += len;
                A::NOP
            }
        }
    }
}

pub struct Get {
    /// Source file path
    src: String,
    fsm: GetFsm
}

impl Get {
    fn new(src: String, read_offset: u64, max_read_offset: u64) -> Get {
        Get { src, fsm: GetFsm::new(read_offset, max_read_offset) }
    }

    fn translate_a(&self, a: GetFsmAction) -> SourceAction<MdxQ, Bytes> {
        fn get_block_locations_request(src: String, offset: u64, length: u64) -> MdxQ {
            let q = pb_cons!{GetBlockLocationsRequestProto,
                src: src,
                offset: offset,
                length: length
            };
            MdxQ::NN(0, None, NnaQ::new(nn::NnQ::GetBlockLocations(q)))
        }

        fn read_block_request(datanode_info: DatanodeInfo, read_block: ReadBlock) -> Result<MdxQ> {
            use std::net::{SocketAddr, IpAddr};
            use std::str::FromStr;

            let (_uuid, ip, port) = datanode_info.into_xfer_info();
            let ip = IpAddr::from_str(&ip).map_err(|e| app_error!(other "Could not parse DN IP `{}`: `{}`", ip, e))?;
            let addr = SocketAddr::new(ip, port as u16);
            Ok(MdxQ::DT(0, Some(addr), dt::DtaQ::ReadBlock(read_block)))
        }

        match a {
            GetFsmAction::NOP =>
                SourceAction::z(),
            GetFsmAction::RequestBlockLocations { offset, len } =>
                SourceAction::z().send(get_block_locations_request(self.src.clone(), offset, len)),
            GetFsmAction::ReadBlock(datanode_info, read_block) =>
                match read_block_request(datanode_info, read_block) {
                    Ok(r) => SourceAction::z().send(r),
                    Err(e) => SourceAction::z().err(e)
                }
            GetFsmAction::Err(e) =>
                SourceAction::z().err(e),
            GetFsmAction::Success =>
                SourceAction::z().eos()
        }
    }

    #[inline]
    fn handle_t(&mut self, evt: GetFsmEvent) -> SourceAction<MdxQ, Bytes> {
        let a = self.fsm.handle(evt);
        self.translate_a(a)
    }

    fn translate_block_locations(&mut self, gblrp: GetBlockLocationsResponseProto) -> Result<Vec<LocatedBlock>> {
        let lbp = pb_decons!(GetBlockLocationsResponseProto, gblrp, locations);
        let (
            file_length, blocks
            //, _under_construction, _last_block, _is_last_block_complete, _file_encryption_info
        ) = pb_decons!(LocatedBlocksProto, lbp,
                    file_length, blocks
                    //, under_construction, last_block, is_last_block_complete, file_encryption_info
                    );

        self.fsm.adjust_max_read_offset(file_length);

        blocks.into_iter().fold(Ok(Vec::new()), |r, block| {
            if let Ok(r) = r {
                let (
                    b, offset, locs, corrupt, block_token //, _is_cached, _storage_types, _storage_ids
                ) = pb_decons!(LocatedBlockProto, block,
                        b, offset, locs, corrupt, block_token //, is_cached, storage_types, storage_ids
                        );
                if corrupt {
                    Err(app_error!(other "All instances of block {:?} are corrupt", b).into())
                } else {
                    Ok(vec_cons(r, LocatedBlock {
                        o: offset,
                        b: b.into(),
                        t: block_token.into(),
                        locs: {
                            let o: Vec<DatanodeInfo> = locs.into_iter().map(|w| w.into()).collect();
                            o.into()
                        }
                    }))
                }
            } else {
                r
            }
        })
    }
}

impl ProtoFsmSource for Get {
    type NQ = MdxQ;
    type NR = MdxR;
    type UR = Bytes;

    fn handle_n(&mut self, ne: NetEvent<MdxR>) -> SourceAction<MdxQ, Bytes> {
        match ne {
            NetEvent::Init =>
                self.handle_t(GetFsmEvent::Init).recv(true),
            NetEvent::Incoming(MdxR::NN(_, NnaR { inner: NnR::GetBlockLocations(gblrp) })) =>
                match self.translate_block_locations(gblrp) {
                    Ok(bloc) => self.handle_t(GetFsmEvent::BlockLocations(bloc)),
                    Err(e) => SourceAction::z().err(e)
                }
            NetEvent::Incoming(MdxR::DT(_, DtaR::Data(bytes))) =>
                if !bytes.is_empty() {
                    self.handle_t(GetFsmEvent::PacketAck(bytes.len() as u64)).deliver(bytes)
                } else {
                    self.handle_t(GetFsmEvent::BlockComplete)
                }
            NetEvent::Incoming(other) =>
                SourceAction::z().err(app_error!(other "Unexpected response: {:?}", other)),
            NetEvent::Idle =>
                SourceAction::z(),
            NetEvent::Err(e) =>
                self.handle_t(GetFsmEvent::Err(e)),
            NetEvent::EndOfStream =>
                SourceAction::z().err(app_error!(premature eof))
        }
    }
}

const MAX_FILE_LENGTH: u64 = std::i64::MAX as u64;


pub type GetStream = source_layer::T<Mdx, Get>;

pub fn get_part_stream(mdx: Mdx, src: String, start_offset: u64, end_offset: u64) -> GetStream {
    source_layer::new(mdx, Get::new(src, start_offset, end_offset))
}

pub fn get_stream(mdx: Mdx, src: String) -> GetStream {
    get_part_stream(mdx, src, 0, MAX_FILE_LENGTH)
}

pub type GetAsyncRead = async_io::AsyncReadStream<GetStream>;

pub fn get(mdx: Mdx, src: String) -> GetAsyncRead {
    async_io::AsyncReadStream::new(get_stream(mdx, src))
}




//--------------------------------------------------------------------------------------------------
// Put command
//--------------------------------------------------------------------------------------------------

struct PutFsm {

}

pub struct Put {
    dst: String
}

impl Put {
    fn new(dst: String) -> Put { Put { dst }}
}


impl ProtoFsmSink for Put {
    type NQ = MdxQ;
    type NR = MdxR;
    type UQ = Bytes;

    fn handle_n(&mut self, ne: NetEvent<MdxR>) -> SinkAction<MdxQ> {
        unimplemented!()
    }

    fn handle_u(&mut self, ue: UserEvent<Bytes>) -> SinkAction<MdxQ> {
        unimplemented!()
    }
}

pub type PutSink = sink_layer::T<Mdx, Put>;

pub fn put_sink(mdx: Mdx, dst: String) -> PutSink {
    sink_layer::new(mdx, Put::new(dst))
}

pub type PutAsyncWrite = async_io::AsyncWriteSink<PutSink>;

pub fn put(mdx: Mdx, dst: String) -> PutAsyncWrite {
    async_io::AsyncWriteSink::new(put_sink(mdx, dst), false)
}