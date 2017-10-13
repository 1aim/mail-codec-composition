use std::ops::Deref;

use error::*;

use external::vec1::Vec1;

use grammar::encoded_word::{
    is_encoded_word,
    EncodedWordContext
};
use super::input::Input;
use super::inner_item::InnerAscii;
use codec::{
    EncodeHandle,
    WriterWrapper, VecWriter,
    base64,
    quoted_printable,
    EncodedWordEncoding
};


#[derive( Debug, Clone, Hash, Eq, PartialEq )]
pub struct EncodedWord {
    inner: InnerAscii,
    ctx: EncodedWordContext
}


impl EncodedWord {

    pub fn write_into<'a, 'b: 'a>(
        handle: &'a mut EncodeHandle<'b>,
        word: &str,
        encoding: EncodedWordEncoding,
        _ctx: EncodedWordContext
    ) {
        //FIXME use the EncodedWordContext
        let mut writer = WriterWrapper::new(
            ascii_str!{ u t f _8 },
            encoding,
            handle
        );
        encoding.encode(word, &mut writer);
    }

    pub fn parse( already_encoded: InnerAscii, ctx: EncodedWordContext ) -> Result<Self> {
        if is_encoded_word( already_encoded.as_str(), ctx ) {
            Ok( EncodedWord { ctx, inner: already_encoded } )
        } else {
            bail!( "the given word is not a valid encoded word in given context: {:?}",
                   &*already_encoded );
        }
    }

    ///
    /// As there is a size limit on encoded words, we might have to split it over multiple
    /// encoded words, therefor we return a vector
    ///
    //TODO use a Vecor which has up to N elements on the stack, this normally is eith 1 or 2
    // of which both can be on the stack
    pub fn encode_word( word: &str, encoding: EncodedWordEncoding, ctx: EncodedWordContext ) -> Vec1<Self> {
        let mut writer = VecWriter::new(ascii_str! { u t f _8 }, encoding);
        encoding.encode( word, &mut writer );
        let vec: Vec1<_> = writer.into();
        let vec = vec.into_iter().map( |ascii| EncodedWord {
            ctx,
            inner: InnerAscii::Owned(ascii)
        }).collect();
        //UNWRAP_SAFE: we can't lose element with a into_iter->map->collect
        Vec1::from_vec(vec)
            .expect( "[BUG] Vec1 -> iter -> map -> Vec1 can not lead to 0 elements" )
    }

    pub fn context( &self ) -> EncodedWordContext {
        self.ctx
    }

    pub fn decode_word( &self ) -> Result<Input> {
        if self.inner.len() < 8 { bail!( "invalid internal encoded word: {:?}", &*self.inner ); }

        let first_question_mark = 1;
        let second_question_mark = self.inner[first_question_mark+1..]
            .as_str()
            .find( "?" )
            .map( |idx| idx + first_question_mark + 1 )
            .ok_or_else( ||-> Error {
                format!("invalid internal encoded word: {:?}", &*self.inner).into()
            })?;

        let third_question_mark = self.inner[second_question_mark+1..]
            .as_str()
            .find( "?" )
            .map( |idx| idx + second_question_mark + 1 )
            .ok_or_else( ||-> Error {
                format!("invalid internal encoded word: {:?}", &*self.inner).into()
            })?;

        let forth_question_mark = self.inner.len() - 2;

        // =?utf8?Q?etcetc?=
        //   ↑   ↑
        let charset = self.inner[first_question_mark+1..second_question_mark].as_str();

        // =?utf8?Q?etcetc?=
        //        ↑↑
        let encoding = self.inner[second_question_mark+1..third_question_mark].as_str();

        // =?utf8?Q?etcetc?=
        //          ↑     ↑
        let data = &self.inner[third_question_mark+1..forth_question_mark];

        //TODO proper charser -> encoder lookup
        if charset != "utf8" {
            //ascii ( and it's official names ) is (for now) not supported,
            // as it's pointless, but will be once there is a proper charser2encod lookup
            // (or to be more concrete given_name => official_name => encoder
            bail!( "unsupported charset in encoded word: {:?}", charset );
        }

        let raw_decoded = match encoding {
            "B" => {
                base64::encoded_word_decode( data )?
            },
            "Q" => {
                quoted_printable::encoded_word_decode( data.as_str() )?
            },
            other => bail!( "unknown encoding: {:?}", other )
        };

        Ok( String::from_utf8( raw_decoded )
            .chain_err( || "found broken encoding in encoded word while decoding" )?
            .into() )

    }
}

impl Deref for EncodedWord {
    type Target = InnerAscii;

    fn deref( &self ) -> &Self::Target {
        &self.inner
    }
}


#[cfg(test)]
mod test {
    use ascii::AsciiString;
    use codec::EncodedWordEncoding;
    use super::*;
    // we do NOT test if encoding/decoding on itself work in this function, it is teste where
    // the function is defined

    #[test]
    fn encode_quoted_printable() {
        let res =
            EncodedWord::encode_word( "täst", EncodedWordEncoding::QuotedPrintable,
                                      EncodedWordContext::Text );

        assert_eq!( 1, res.len() );
        assert_eq!(
            "=?utf8?Q?t=C3=A4st?=",
            &*res[0].inner
        );
    }

    #[test]
    fn encode_base64() {
        let res =
            EncodedWord::encode_word( "täst", EncodedWordEncoding::Base64,
                                      EncodedWordContext::Text );

        assert_eq!( 1, res.len() );
        assert_eq!(
            "=?utf8?B?dMOkc3Q=?=",
            &*res[0].inner
        );
    }

    #[test]
    fn parse() {
        //NOTE: parse rellys havily on is_encoded_word, which is tested in `::grammar::encoded_word`
        let asciied = AsciiString::from_ascii( "=?utf8?Q?123?=" ).unwrap();
        let ec_res = EncodedWord::parse( asciied.into(), EncodedWordContext::Text );
        assert_eq!( true, ec_res.is_ok() );
        let ec = ec_res.unwrap();
        assert_eq!(
            "=?utf8?Q?123?=",
            &*ec.inner
        )
    }

    #[test]
    fn parse_err() {
        //NOTE: parse rellys havily on is_encoded_word, which is tested in `::grammar::encoded_word`
        let asciied = AsciiString::from_ascii( "=?utf8???Q123?=" ).unwrap();
        let ec_res = EncodedWord::parse( asciied.into() , EncodedWordContext::Text );
        assert_eq!( false, ec_res.is_ok() );
    }

    #[test]
    fn decode_base64() {
        let asciied = AsciiString::from_ascii( "=?utf8?B?dMOkc3Q=?=" ).unwrap();
        let ec = EncodedWord::parse( asciied.into(), EncodedWordContext::Text ).unwrap();
        let dec = ec.decode_word().unwrap();
        assert_eq!(
            "täst",
            &**dec
        );
    }

    #[test]
    fn decode_quoted_printable() {
        let asciied = AsciiString::from_ascii(  "=?utf8?Q?t=C3=A4st?=" ).unwrap();
        let ec = EncodedWord::parse( asciied.into(), EncodedWordContext::Text ).unwrap();
        let dec = ec.decode_word().unwrap();
        assert_eq!(
            "täst",
            &**dec
        );
    }

    #[test]
    fn unknow_encoding() {
        let asciied = AsciiString::from_ascii( "=?utf8?R?test?=" ).unwrap();
        let ec = EncodedWord::parse( asciied.into(), EncodedWordContext::Text ).unwrap();
        assert_eq!( false, ec.decode_word().is_ok() );
    }

    #[test]
    fn broken_encoding() {
        let asciied = AsciiString::from_ascii( "=?utf8?Q?ab=_ups?=" ).unwrap();
        let ec = EncodedWord::parse( asciied.into(), EncodedWordContext::Text ).unwrap();
        assert_eq!( false, ec.decode_word().is_ok() );
    }

    #[test]
    fn broken_charset_encoding() {
        let asciied = AsciiString::from_ascii( "=?utf8?Q?ab=FFups?=" ).unwrap();
        let ec = EncodedWord::parse( asciied.into(), EncodedWordContext::Text ).unwrap();
        assert_eq!( false, ec.decode_word().is_ok() );
    }

    #[test]
    fn multi_char_encoding() {
        let asciied = AsciiString::from_ascii( "=?utf8?Qnot?abcd?=" ).unwrap();
        let res = EncodedWord::parse( asciied.into() , EncodedWordContext::Text );
        assert_eq!( true, res.is_ok() );
        let dec_res = res.unwrap().decode_word();
        assert_eq!( false, dec_res.is_ok() );
    }
    //TODO tests: [long word => multiple word], [is context used]

//    #[test]
//    fn long_word_splitting() {
//
//    }

//    #[test]
//    fn uses_context_text() {
//
//    }
//
//    #[test]
//    fn uses_context_phrase() {
//
//    }
//
//    #[test]
//    fn uses_context_comment() {
//
//    }


}