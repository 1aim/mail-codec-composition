use mime::Mime;
use base64;
use quoted_printable;


#[allow(unused_doc_comment)]
error_chain! {

    foreign_links {
        DecodeBase64(base64::DecodeError);
        DecodeQuotedPrintable(quoted_printable::QuotedPrintableError);
    }


    errors {

        /// Certain components might not be encodable under some circumstances.
        /// E.g. they might have non-ascii values and are not encodable into ascii
        ///
        /// a example for this would be a non ascii `local-part` of `addr-spec`
        /// (i.e. the part of a email address befor the `@`)
        NonEncodableComponents( component: &'static str, data: String ) {
            description( "given information can not be encoded into ascii" )
            display( "can not encode the {} component with value {:?}", component, data )
        }

        TriedWriting8BitBytesInto7BitData {
            description(
                "the program tried to write a non ascii string while smtputf8 was not supported" )
        }

        AtLastOneElementIsRequired {
            description( concat!( "for the operation a list with at last one element",
                                  " is required but and empty list was given" ) )
        }

        InvalidHeaderName(name: String) {
            description( "given header name is not valid" )
            display( "{:?} is not a valid header name", name )
        }

        NotMultipartMime( mime: Mime ) {
            description( "expected a multipart mime for a multi part body" )
            display( _self ) -> ( "{}, got: {}", _self.description(), mime )
        }

        MultipartBoundaryMissing {
            description( "multipart boundary is missing" )
        }

        NotSinglepartMime( mime: Mime ) {
            description( "expected a non-multipart mime for a non-multipart body" )
            display( _self ) -> ( "{}, got: {}", _self.description(), mime )
        }

        NeedPlainAndOrHtmlMailBody {

        }

        ContentTypeAndBodyIncompatible {
            description( concat!(
                "given content type is incompatible with body,",
                "e.g. using a non multipart mime with a multipart body" ) )
        }

        UnknownTransferEncoding( encoding: String ) {
            description( "the given transfer encoding is not supported" )
            display( "the transfer encoding {:?} is not supported", encoding )
        }

        Invalide7BitValue( byte: u8 ) {
            description( "the byte is not valid in 7bit (content transfer) encoding" )
        }
        Invalide8BitValue( val: u8 ) {
            description( "the byte is not valid in 8bit (content transfer) encoding" )
        }

        Invalide7BitSeq( byte: u8 ) {
            description( "the chars '\\r', '\\n' can only appear as \"\\r\\n\" in 7bit (content transfer) encoding " )
        }
        Invalide8BitSeq( val: u8 ) {
            description( "the chars '\\r', '\\n' can only appear as \"\\r\\n\" in 8bit (content transfer) encoding " )
        }

        BodyFutureResolvedToAnError {
        
        }

        NeedAtLastOneBodyInMultipartMail {

        }

        GeneratingMimeFailed {

        }

        RegisterExtensionsToLate( extension: String ) {
            description( "can not register extensions after Store/Look-Up-Table was generated" )
        }
    }
}