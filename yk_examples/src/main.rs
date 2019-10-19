
extern crate yk_lexer;
extern crate yk_parser;

use yk_lexer::{TokenType, Lexer};
use yk_parser::{yk_parser, ParseResult};

#[derive(yk_lexer::Lexer, PartialEq, Eq, Debug)]
enum TokTy {
    #[error]
    Error,

    #[end]
    End,

    #[regex(r"[ \r\n]")]
    #[ignore]
    Whitespace,

    #[c_ident]
    Ident,

    #[regex("[0-9]+")]
    IntLit,

    #[token("+")]
    Add,

    #[token("-")]
    Sub,

    #[token("*")]
    Mul,

    #[token("/")]
    Div,

    #[token("^")]
    Exp,

    #[token("(")]
    LP,

    #[token(")")]
    RP,
}

mod parser {
use :: yk_parser :: { irec , drec , ParseResult , ParseOk , ParseErr } ; use
:: std :: string :: String ; use :: std :: option :: Option ; use :: std ::
collections :: HashMap ; use :: std :: iter :: Iterator ; use :: std :: clone
:: Clone ; use :: std :: cmp :: { PartialEq , Eq } ; use :: std :: hash ::
Hash ; use :: std :: boxed :: Box ; use :: std :: rc :: Rc ; use :: std :: fmt
:: Display ; pub struct MemoContext < I > {
call_stack : irec :: CallStack , call_heads : irec :: CallHeadTable ,
memo_ones : HashMap < usize , irec :: Entry < I , ( i32 ) > > , memo_ones_impl
: HashMap < usize , irec :: Entry < I , ( i32 ) > > } impl < I > MemoContext <
I > {
pub fn new (  ) -> Self {
Self {
call_stack : irec :: CallStack :: new (  ) , call_heads : irec ::
CallHeadTable :: new (  ) , memo_ones : HashMap :: new (  ) , memo_ones_impl :
HashMap :: new (  ) } } } fn insert_and_get < K , V > (
m : & mut HashMap < K , V > , k : K , v : V ) -> & V where K : Clone + Eq +
Hash { m . insert ( k . clone (  ) , v ) ; m . get ( & k ) . unwrap (  ) } fn
recall_ones < I > ( memo : & mut MemoContext < I > , src : I , idx : usize )
-> Option < irec :: Entry < I , ( i32 ) >> where I : Iterator + Clone , < I as
Iterator > :: Item : PartialEq < char > + Display , I : 'static , ( i32 ) :
'static {
let curr_rule = "ones" ; let cached = memo . memo_ones . get ( & idx ) ; let
in_heads = memo . call_heads . get_mut ( & idx ) ; match ( in_heads , cached )
{
( None , None ) => None , ( None , Some ( c ) ) => Some ( ( * c ) . clone (  )
) , ( Some ( h ) , c ) => {
if c . is_none (  ) && ! (
"ones" == h . head || h . involved . contains ( "ones" ) ) {
Some ( irec :: Entry :: ParseResult ( ParseErr :: new (  ) . into (  ) ) ) }
else if Rc :: get_mut ( h ) . unwrap (  ) . eval . remove ( "ones" ) {
let tmp_res = { parse_ones_impl ( memo , src . clone (  ) , idx ) } ; Some (
insert_and_get (
& mut memo . memo_ones , idx , irec :: Entry :: ParseResult ( tmp_res ) ) .
clone (  ) ) } else { c . cloned (  ) } } } } fn lr_answer_ones < I > (
memo : & mut MemoContext < I > , src : I , idx : usize , growable : & mut irec
:: LeftRecursive ) -> ParseResult < I , ( i32 ) > where I : Iterator + Clone ,
< I as Iterator > :: Item : PartialEq < char > + Display , I : 'static , ( i32
) : 'static {
assert ! ( growable . head . is_some (  ) ) ; let seed = growable .
parse_result (  ) . unwrap (  ) . clone (  ) ; let head_rc = growable . head .
as_mut (  ) . unwrap (  ) ; {
let head = Rc :: get_mut ( head_rc ) . unwrap (  ) ; if head . head != "ones"
{ return seed ; } } let s = insert_and_get (
& mut memo . memo_ones , idx , irec :: Entry :: ParseResult ( seed ) ) .
parse_result (  ) . clone (  ) ; if s . is_err (  ) { return s ; } else {
return grow_ones ( memo , src , idx , s , head_rc ) ; } } fn grow_ones < I > (
memo : & mut MemoContext < I > , src : I , idx : usize , old : ParseResult < I
, ( i32 ) > , h : & mut Rc < irec :: RecursionHead > ) -> ParseResult < I , (
i32 ) > where I : Iterator + Clone , < I as Iterator > :: Item : PartialEq <
char > + Display , I : 'static , ( i32 ) : 'static {
let curr_rule = "ones" ; memo . call_heads . insert ( idx , h . clone (  ) ) ;
Rc :: get_mut ( h ) . unwrap (  ) . eval = h . involved . clone (  ) ; let
tmp_res = { parse_ones_impl ( memo , src . clone (  ) , idx ) } ; if tmp_res .
is_ok (  ) && old . furthest_look (  ) < tmp_res . furthest_look (  ) {
let new_old = insert_and_get (
& mut memo . memo_ones , idx , irec :: Entry :: ParseResult ( tmp_res ) ) .
parse_result (  ) . clone (  ) ; return grow_ones (
memo , src , idx , new_old , h ) ; } memo . call_heads . remove ( & idx ) ;
let updated = ParseResult :: unify_alternatives ( tmp_res , old ) ; return
insert_and_get (
& mut memo . memo_ones , idx , irec :: Entry :: ParseResult ( updated ) ) .
parse_result (  ) . clone (  ) ; } pub fn parse_ones < I > (
memo : & mut MemoContext < I > , src : I , idx : usize ) -> ParseResult < I ,
( i32 ) > where I : Iterator + Clone , < I as Iterator > :: Item : PartialEq <
char > + Display , I : 'static , ( i32 ) : 'static {
let curr_rule = "ones" ; let m = recall_ones ( memo , src . clone (  ) , idx )
; match m {
None => {
let mut base = Rc :: new (
irec :: LeftRecursive :: with_parser_and_seed :: < I , ( i32 ) > (
"ones" , ParseErr :: new (  ) . into (  ) ) ) ; memo . call_stack . push (
base . clone (  ) ) ; memo . memo_ones . insert (
idx , irec :: Entry :: LeftRecursive ( base . clone (  ) ) ) ; let tmp_res = {
parse_ones_impl ( memo , src . clone (  ) , idx ) } ; memo . call_stack . pop
(  ) ; if base . head . is_none (  ) {
insert_and_get (
& mut memo . memo_ones , idx , irec :: Entry :: ParseResult ( tmp_res ) ) .
parse_result (  ) . clone (  ) } else {
Rc :: get_mut ( & mut base ) . unwrap (  ) . seed = Box :: new ( tmp_res ) ;
lr_answer_ones (
memo , src . clone (  ) , idx , Rc :: get_mut ( & mut base ) . unwrap (  ) ) }
} , Some ( irec :: Entry :: LeftRecursive ( mut lr ) ) => {
memo . call_stack . setup_lr (
"ones" , Rc :: get_mut ( & mut lr ) . unwrap (  ) ) ; lr . parse_result (  ) .
unwrap (  ) . clone (  ) } , Some ( irec :: Entry :: ParseResult ( r ) ) => {
r . clone (  ) } } } fn recall_ones_impl < I > (
memo : & mut MemoContext < I > , src : I , idx : usize ) -> Option < irec ::
Entry < I , ( i32 ) >> where I : Iterator + Clone , < I as Iterator > :: Item
: PartialEq < char > + Display , I : 'static , ( i32 ) : 'static {
let curr_rule = "ones_impl" ; let cached = memo . memo_ones_impl . get ( & idx
) ; let in_heads = memo . call_heads . get_mut ( & idx ) ; match (
in_heads , cached ) {
( None , None ) => None , ( None , Some ( c ) ) => Some ( ( * c ) . clone (  )
) , ( Some ( h ) , c ) => {
if c . is_none (  ) && ! (
"ones_impl" == h . head || h . involved . contains ( "ones_impl" ) ) {
Some ( irec :: Entry :: ParseResult ( ParseErr :: new (  ) . into (  ) ) ) }
else if Rc :: get_mut ( h ) . unwrap (  ) . eval . remove ( "ones_impl" ) {
let tmp_res = {
{
let res1 = {
{
let res = {
{
let res1 = { parse_ones ( memo , src . clone (  ) , idx ) } ; if let
ParseResult :: Ok ( ok ) = res1 {
let src = ok . furthest_it . clone (  ) ; let idx = ok . matched ; let res2 =
{
{
let mut src2 = src . clone (  ) ; if let Some ( v ) = src2 . next (  ) {
if v == '1' {
ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  ) } else {
let got = format ! ( "{}" , v ) ; ParseErr :: single (
idx , got , curr_rule , "\'1\'" . into (  ) ) . into (  ) } } else {
ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , "\'1\'" . into (  ) ) . into (
) } } } ; let res_tmp = ParseResult :: unify_sequence ( ok , res2 ) ; if let
ParseResult :: Ok ( ok ) = res_tmp {
ok . map ( | ( ( e0 ) , ( e1 ) ) | ( e0 , e1 ) ) . into (  ) } else {
res_tmp . err (  ) . unwrap (  ) . into (  ) } } else {
res1 . err (  ) . unwrap (  ) . into (  ) } } } ; if let ParseResult :: Ok (
ok ) = res {
ok . map ( | ( e0 , e1 ) | ( | e0 , e1 | { e0 + 1 } ) ( e0 , e1 ) ) . into (
) } else { res . err (  ) . unwrap (  ) . into (  ) } } } ; let res2 = {
{
let res = {
{
let mut src2 = src . clone (  ) ; if let Some ( v ) = src2 . next (  ) {
if v == '1' {
ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  ) } else {
let got = format ! ( "{}" , v ) ; ParseErr :: single (
idx , got , curr_rule , "\'1\'" . into (  ) ) . into (  ) } } else {
ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , "\'1\'" . into (  ) ) . into (
) } } } ; if let ParseResult :: Ok ( ok ) = res {
ok . map ( | ( e0 ) | ( | e0 | { 1 } ) ( e0 ) ) . into (  ) } else {
res . err (  ) . unwrap (  ) . into (  ) } } } ; ParseResult ::
unify_alternatives ( res1 , res2 ) } } ; Some (
insert_and_get (
& mut memo . memo_ones_impl , idx , irec :: Entry :: ParseResult ( tmp_res ) )
. clone (  ) ) } else { c . cloned (  ) } } } } fn lr_answer_ones_impl < I > (
memo : & mut MemoContext < I > , src : I , idx : usize , growable : & mut irec
:: LeftRecursive ) -> ParseResult < I , ( i32 ) > where I : Iterator + Clone ,
< I as Iterator > :: Item : PartialEq < char > + Display , I : 'static , ( i32
) : 'static {
assert ! ( growable . head . is_some (  ) ) ; let seed = growable .
parse_result (  ) . unwrap (  ) . clone (  ) ; let head_rc = growable . head .
as_mut (  ) . unwrap (  ) ; {
let head = Rc :: get_mut ( head_rc ) . unwrap (  ) ; if head . head !=
"ones_impl" { return seed ; } } let s = insert_and_get (
& mut memo . memo_ones_impl , idx , irec :: Entry :: ParseResult ( seed ) ) .
parse_result (  ) . clone (  ) ; if s . is_err (  ) { return s ; } else {
return grow_ones_impl ( memo , src , idx , s , head_rc ) ; } } fn
grow_ones_impl < I > (
memo : & mut MemoContext < I > , src : I , idx : usize , old : ParseResult < I
, ( i32 ) > , h : & mut Rc < irec :: RecursionHead > ) -> ParseResult < I , (
i32 ) > where I : Iterator + Clone , < I as Iterator > :: Item : PartialEq <
char > + Display , I : 'static , ( i32 ) : 'static {
let curr_rule = "ones_impl" ; memo . call_heads . insert (
idx , h . clone (  ) ) ; Rc :: get_mut ( h ) . unwrap (  ) . eval = h .
involved . clone (  ) ; let tmp_res = {
{
let res1 = {
{
let res = {
{
let res1 = { parse_ones ( memo , src . clone (  ) , idx ) } ; if let
ParseResult :: Ok ( ok ) = res1 {
let src = ok . furthest_it . clone (  ) ; let idx = ok . matched ; let res2 =
{
{
let mut src2 = src . clone (  ) ; if let Some ( v ) = src2 . next (  ) {
if v == '1' {
ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  ) } else {
let got = format ! ( "{}" , v ) ; ParseErr :: single (
idx , got , curr_rule , "\'1\'" . into (  ) ) . into (  ) } } else {
ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , "\'1\'" . into (  ) ) . into (
) } } } ; let res_tmp = ParseResult :: unify_sequence ( ok , res2 ) ; if let
ParseResult :: Ok ( ok ) = res_tmp {
ok . map ( | ( ( e0 ) , ( e1 ) ) | ( e0 , e1 ) ) . into (  ) } else {
res_tmp . err (  ) . unwrap (  ) . into (  ) } } else {
res1 . err (  ) . unwrap (  ) . into (  ) } } } ; if let ParseResult :: Ok (
ok ) = res {
ok . map ( | ( e0 , e1 ) | ( | e0 , e1 | { e0 + 1 } ) ( e0 , e1 ) ) . into (
) } else { res . err (  ) . unwrap (  ) . into (  ) } } } ; let res2 = {
{
let res = {
{
let mut src2 = src . clone (  ) ; if let Some ( v ) = src2 . next (  ) {
if v == '1' {
ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  ) } else {
let got = format ! ( "{}" , v ) ; ParseErr :: single (
idx , got , curr_rule , "\'1\'" . into (  ) ) . into (  ) } } else {
ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , "\'1\'" . into (  ) ) . into (
) } } } ; if let ParseResult :: Ok ( ok ) = res {
ok . map ( | ( e0 ) | ( | e0 | { 1 } ) ( e0 ) ) . into (  ) } else {
res . err (  ) . unwrap (  ) . into (  ) } } } ; ParseResult ::
unify_alternatives ( res1 , res2 ) } } ; if tmp_res . is_ok (  ) && old .
furthest_look (  ) < tmp_res . furthest_look (  ) {
let new_old = insert_and_get (
& mut memo . memo_ones_impl , idx , irec :: Entry :: ParseResult ( tmp_res ) )
. parse_result (  ) . clone (  ) ; return grow_ones_impl (
memo , src , idx , new_old , h ) ; } memo . call_heads . remove ( & idx ) ;
let updated = ParseResult :: unify_alternatives ( tmp_res , old ) ; return
insert_and_get (
& mut memo . memo_ones_impl , idx , irec :: Entry :: ParseResult ( updated ) )
. parse_result (  ) . clone (  ) ; } pub fn parse_ones_impl < I > (
memo : & mut MemoContext < I > , src : I , idx : usize ) -> ParseResult < I ,
( i32 ) > where I : Iterator + Clone , < I as Iterator > :: Item : PartialEq <
char > + Display , I : 'static , ( i32 ) : 'static {
let curr_rule = "ones_impl" ; let m = recall_ones_impl (
memo , src . clone (  ) , idx ) ; match m {
None => {
let mut base = Rc :: new (
irec :: LeftRecursive :: with_parser_and_seed :: < I , ( i32 ) > (
"ones_impl" , ParseErr :: new (  ) . into (  ) ) ) ; memo . call_stack . push
( base . clone (  ) ) ; memo . memo_ones_impl . insert (
idx , irec :: Entry :: LeftRecursive ( base . clone (  ) ) ) ; let tmp_res = {
{
let res1 = {
{
let res = {
{
let res1 = { parse_ones ( memo , src . clone (  ) , idx ) } ; if let
ParseResult :: Ok ( ok ) = res1 {
let src = ok . furthest_it . clone (  ) ; let idx = ok . matched ; let res2 =
{
{
let mut src2 = src . clone (  ) ; if let Some ( v ) = src2 . next (  ) {
if v == '1' {
ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  ) } else {
let got = format ! ( "{}" , v ) ; ParseErr :: single (
idx , got , curr_rule , "\'1\'" . into (  ) ) . into (  ) } } else {
ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , "\'1\'" . into (  ) ) . into (
) } } } ; let res_tmp = ParseResult :: unify_sequence ( ok , res2 ) ; if let
ParseResult :: Ok ( ok ) = res_tmp {
ok . map ( | ( ( e0 ) , ( e1 ) ) | ( e0 , e1 ) ) . into (  ) } else {
res_tmp . err (  ) . unwrap (  ) . into (  ) } } else {
res1 . err (  ) . unwrap (  ) . into (  ) } } } ; if let ParseResult :: Ok (
ok ) = res {
ok . map ( | ( e0 , e1 ) | ( | e0 , e1 | { e0 + 1 } ) ( e0 , e1 ) ) . into (
) } else { res . err (  ) . unwrap (  ) . into (  ) } } } ; let res2 = {
{
let res = {
{
let mut src2 = src . clone (  ) ; if let Some ( v ) = src2 . next (  ) {
if v == '1' {
ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  ) } else {
let got = format ! ( "{}" , v ) ; ParseErr :: single (
idx , got , curr_rule , "\'1\'" . into (  ) ) . into (  ) } } else {
ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , "\'1\'" . into (  ) ) . into (
) } } } ; if let ParseResult :: Ok ( ok ) = res {
ok . map ( | ( e0 ) | ( | e0 | { 1 } ) ( e0 ) ) . into (  ) } else {
res . err (  ) . unwrap (  ) . into (  ) } } } ; ParseResult ::
unify_alternatives ( res1 , res2 ) } } ; memo . call_stack . pop (  ) ; if
base . head . is_none (  ) {
insert_and_get (
& mut memo . memo_ones_impl , idx , irec :: Entry :: ParseResult ( tmp_res ) )
. parse_result (  ) . clone (  ) } else {
Rc :: get_mut ( & mut base ) . unwrap (  ) . seed = Box :: new ( tmp_res ) ;
lr_answer_ones_impl (
memo , src . clone (  ) , idx , Rc :: get_mut ( & mut base ) . unwrap (  ) ) }
} , Some ( irec :: Entry :: LeftRecursive ( mut lr ) ) => {
memo . call_stack . setup_lr (
"ones_impl" , Rc :: get_mut ( & mut lr ) . unwrap (  ) ) ; lr . parse_result (
 ) . unwrap (  ) . clone (  ) } , Some ( irec :: Entry :: ParseResult ( r ) )
=> { r . clone (  ) } } } }

fn main() {
    let src = "1111";

    let r = parser::parse_ones(&mut parser::MemoContext::new(), src.chars(), 0);
    if r.is_ok() {
        let val = r.ok().unwrap().value;
        println!("Ok: {:?}", val);
    }
    else {
        let err = r.err().unwrap();
        println!("Err:");
        for (rule, element) in err.elements {
            print!("  While parsing {} expected: ", rule);

            let mut fst = true;
            for tok in element.expected_elements {
                if !fst {
                    print!(" or ");
                }
                fst = false;
                print!("{}", tok);
            }
            println!();
        }
        println!("But got '{}'", err.found_element);
    }

    /*
    // Creating a lexer
    let mut lexer = MyTokenType::lexer();
    let mut tokens = Vec::new();
    // Modify
    let m = lexer.modify(&tokens, 0..0, "hello world");
    tokens.splice(m.erased, m.inserted);
    print_tokens(lexer.source(), &tokens);
    // Modify
    let m = lexer.modify(&tokens, 5..5, " there");
    tokens.splice(m.erased, m.inserted);
    print_tokens(lexer.source(), &tokens);
    */
}
