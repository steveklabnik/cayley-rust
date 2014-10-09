use std::str;
use std::io::println;
use std::io::Stream;

use url::Url;
use http::client::RequestWriter;
use http::method::Post;
use http::headers::HeaderEnum;

use serialize::{Decoder, Decodable};
use serialize::json::decode as json_decode;
use serialize::json::DecoderError;

use std::collections::HashMap;

use path::Query;
use path::Reuse;

use errors::{GraphRequestError, GraphResult,
             InvalidUrl, MalformedRequest, RequestFailed,
             DecodingFailed, ResponseParseFailed,
             QueryNotFinalized, QueryCompilationFailed,
             ReusableCannotBeSaved };

pub struct Graph {
    url: String,
    request: Box<RequestWriter>
}

pub struct GraphNode<'gn>(pub HashMap<&'gn str, &'gn str>);

pub struct GraphNodes<'gns>(pub Vec<GraphNode<'gns>>);

pub enum CayleyAPIVersion { V1, DefaultVersion }

impl Graph {

    pub fn default() -> GraphResult<Graph> {
        Graph::new("localhost", 64210, DefaultVersion)
    }

    pub fn new(host: &str, port: int, version: CayleyAPIVersion) -> GraphResult<Graph> {
        let version_str = match version { V1 | DefaultVersion => "v1" };
        let url = format!("http://{:s}:{:d}/api/{:s}/query/gremlin",
                          host, port, version_str);
        match Graph::prepare_request(url.as_slice()) {
            Ok(request) => { /* TODO: match request.try_connect() */
                             let mut path: Vec<String> = Vec::with_capacity(20);
                             path.push("graph".to_string());
                             Ok(Graph{ url: url,
                                       request: box request }) },
            Err(error) => Err(error)
        }
    }

    // find nodes by query implementation and return them parsed
    pub fn find<'gns>(self, query: &Query) -> GraphResult<GraphNodes<'gns>> {
        match query.is_finalized() {
            true => match query.compile() {
                Some(compiled) => self.find_by(compiled),
                None => Err(QueryCompilationFailed)
            },
            false => Err(QueryNotFinalized)
        }
    }

    // find nodes using raw pre-compiled string query and return them parsed
    pub fn find_by<'gns>(self, query: String) -> GraphResult<GraphNodes<'gns>> {
        match Graph::perform_request(self.request, query) {
            Ok(body) => Graph::decode_nodes(body),
            Err(error) => Err(error)
        }
    }

    pub fn save(self, reusable: &mut Reuse) -> GraphResult<()> {
        match reusable.save() {
            Some(query) => {
                match Graph::perform_request(self.request, query) {
                    Ok(body) => { reusable.set_saved(); Ok(()) },
                    Err(error) => Err(error)
                }
            },
            None => Err(ReusableCannotBeSaved)
        }
    }

    pub fn save_as(self, name: &str, reusable: &mut Reuse) -> GraphResult<()> {
        match reusable.save_as(name) {
            Some(query) => {
                match Graph::perform_request(self.request, query) {
                    Ok(body) => { reusable.set_saved(); Ok(()) },
                    Err(error) => Err(error)
                }
            },
            None => Err(ReusableCannotBeSaved)
        }
    }

    // prepares the RequestWriter object from URL to save it inside the Graph for future re-use
    fn prepare_request(url: &str) -> GraphResult<RequestWriter> {
        match Url::parse(url) {
            Err(error) => Err(InvalidUrl(error, url.to_string())),
            Ok(parsed_url) => {
                match RequestWriter::new(Post, parsed_url) {
                    Err(error) => Err(MalformedRequest(error, url.to_string())),
                    Ok(request) => Ok(request)
                }
            }
        }
    }

    // uses RequestWriter to perform a request with given request body and returns the response body
    fn perform_request(mut request: Box<RequestWriter>, body: String) -> GraphResult<Vec<u8>> {
        request.headers.content_length = Some(body.len());
        match request.write_str(body.as_slice()) {
            Err(error) => Err(RequestFailed(error, body)),
            Ok(_) => match request.read_response() {
                Err((_, error)) => Err(RequestFailed(error, body)),
                Ok(mut response) => match response.read_to_end() {
                    Err(error) => Err(RequestFailed(error, body)),
                    Ok(response_body) => Ok(response_body)
                }
            }
        }
    }

    // extract JSON nodes from response
    fn decode_nodes<'gns>(source: Vec<u8>) -> GraphResult<GraphNodes<'gns>> {
        match str::from_utf8(source.as_slice()) {
            None => Err(ResponseParseFailed),
            Some(nodes_json) => {
                match json_decode(nodes_json) {
                    Err(error) => Err(DecodingFailed(error, nodes_json.to_string())),
                    Ok(nodes) => Ok(nodes)
                }
            }
        }
    }

}

/* impl<'gn> GraphNode<'gn> {

    fn new<'gn>(init: HashMap<&'gn str, &'gn str>) -> GraphNode<'gn> {
        GraphNode{ data: init }
    }

} */

impl<'gn, S: Decoder<E>, E> Decodable<S, E> for GraphNode<'gn> {
    fn decode(decoder: &mut S) -> Result<GraphNode<'gn>, E> {
        /* match Decodable::decode(decoder) {
            Ok(map) => GraphNode(map),
            Err(err) => return Err(err)
        } */
        Decodable::decode(decoder)
    }
}

impl<'gns, S: Decoder<E>, E> Decodable<S, E> for GraphNodes<'gns> {
    fn decode(decoder: &mut S) -> Result<GraphNodes<'gns>, E> {
        decoder.read_struct("__unused__", 0, |decoder| {
            decoder.read_struct_field("result", 0, |decoder| {
                decoder.read_seq(|decoder, len| {
                    let mut nodes: Vec<GraphNode> = Vec::with_capacity(len);
                    for i in range(0u, len) {
                        nodes.push(match decoder.read_seq_elt(i,
                                        |decoder| { Decodable::decode(decoder) }) {
                            Ok(node) => node,
                            Err(err) => return Err(err)
                        });
                    };
                    Ok(GraphNodes(nodes))
                })
            })
        })
    }
}
