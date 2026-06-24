
/*

    AUTO SIMD LOOP 
    
    Purpose:
    This is a compile time syntax sugar designed to make loop is auto SIMD friendly. It also has guard that will return compile time error if branching code (if and match) is detected because it prevents auto SIMD
    
    Some simple `if`0 will be compiled to cmov that is branchless so SIMD friendly, eg let a = if condition { b } else { c };
    But for `if` that calls/does complex code will be compiled to jump instruction, it is branching, that makes auto SIMD is not used
    Because I haven't find how to detect them accurately, as a result, this macro rejects any kind of if
    Use branchless programming techniques instead, like bitwise or arithmetic operations
    
*/

#[macro_export]
macro_rules! __cek_if_match {
    (if $($tail:tt)*) => { 
        compile_error!(r#"
 LOOP ERROR  ->  Forbidden 'if' keyword detected!
 REASON      ->  Branching breaks auto vectorization.
 SOLUTION    ->  Use branchless masks or arithmetic multiplication instead.
"#); 
    };
    (match $($tail:tt)*) => { 
        compile_error!(r#"
 LOOP ERROR  ->  Forbidden 'match' keyword detected!
 REASON      ->  Pattern matching disrupts contiguous SIMD execution.
 SOLUTION    ->  Use branchless masks or arithmetic multiplication instead.
"#); 
    };
    ({ $($inner:tt)* } $($tail:tt)*) => { __cek_if_match!($($inner)*); __cek_if_match!($($tail)*); };
    ([ $($inner:tt)* ] $($tail:tt)*) => { __cek_if_match!($($inner)*); __cek_if_match!($($tail)*); };
    (( $($inner:tt)* ) $($tail:tt)*) => { __cek_if_match!($($inner)*); __cek_if_match!($($tail)*); };
    ($t:tt $($tail:tt)*) => { __cek_if_match!($($tail)*); };
    () => {};
}

#[macro_export]
macro_rules! __get_first_len {
    (($is_mut:ident $head:expr) $(, $rest:tt)*) => { $head.len() };
}

#[macro_export]
macro_rules! __make_simd_chunk_iter {
    (mut, $slice:expr, $simd_len:expr, $total_chunk:expr) => { $slice.split_at_mut($simd_len).0.chunks_exact_mut($total_chunk) };
    (const, $slice:expr, $simd_len:expr, $total_chunk:expr) => { $slice.split_at($simd_len).0.chunks_exact($total_chunk) };
}

#[macro_export]
macro_rules! __make_scalar_elem_iter {
    (mut, $slice:expr, $simd_len:expr) => { $slice.split_at_mut($simd_len).1.iter_mut() };
    (const, $slice:expr, $simd_len:expr) => { $slice.split_at($simd_len).1.iter() };
}

#[macro_export]
macro_rules! __make_sub_chunk_iter {
    (mut, $large:expr, $size:expr) => { $large.chunks_exact_mut($size) };
    (const, $large:expr, $size:expr) => { $large.chunks_exact($size) };
}

#[macro_export]
macro_rules! __as_array {
    (mut, $c:expr, $size:expr) => { { let r: &mut [_; $size] = $c.try_into().unwrap(); r } };
    (const, $c:expr, $size:expr) => { { let r: &[_; $size] = $c.try_into().unwrap(); r } };
}

#[macro_export]
macro_rules! __make_elem_iter {
    (mut, $chunk:expr) => { $chunk.iter_mut() };
    (const, $chunk:expr) => { $chunk.iter() };
}

#[macro_export]
macro_rules! zip_iters {
    ($a:expr, $b:expr $(, $rest:expr)*) => { zip_iters!($a.zip($b) $(, $rest)*) };
    ($zipped:expr) => { $zipped };
}

#[macro_export]
macro_rules! zip_pat {
    ($a:pat, $b:pat $(, $rest:pat)*) => { zip_pat!(($a, $b) $(, $rest)*) };
    ($p:pat) => { $p };
}

#[macro_export]
macro_rules! __simd_exec {
    ($chunk_size:expr, $unroll:expr, $idx:ident, [$( ($is_mut:ident $slice:expr , $ident:ident) ),*], { $($body:tt)* }, $pre:block, $post:block) => {{
        #[allow(unused_variables, unused_assignments)]
        {
            __cek_if_match!($($body)*);
            let total_chunk = $chunk_size * $unroll;
            let first_len = __get_first_len!( $( ($is_mut $slice) ),* );
            let simd_len = (first_len / total_chunk) * total_chunk;

            let mut $idx = 0;

            let mut large_chunks_zipped = zip_iters!( $( __make_simd_chunk_iter!($is_mut, $slice, simd_len, total_chunk) ),* );
            while let Some(zip_pat!( $($ident),* )) = large_chunks_zipped.next() {
                $pre
                let mut sub_chunks_zipped = zip_iters!( $( __make_sub_chunk_iter!($is_mut, $ident, $chunk_size) ),* );
                while let Some(zip_pat!( $($ident),* )) = sub_chunks_zipped.next() {
                    $(
                        let $ident = __as_array!($is_mut, $ident, $chunk_size);
                    )*
                    let mut elems_zipped = zip_iters!( $( __make_elem_iter!($is_mut, $ident) ),* );
                    while let Some(zip_pat!( $($ident),* )) = elems_zipped.next() {
                        $($body)*
                        $idx += 1;
                    }
                }
                $post
            }

            $pre
            let mut scalar_elems_zipped = zip_iters!( $( __make_scalar_elem_iter!($is_mut, $slice, simd_len) ),* );
            while let Some(zip_pat!( $($ident),* )) = scalar_elems_zipped.next() {
                $($body)*
                $idx += 1;
            }
            $post
        }
    }};
}

#[macro_export]
macro_rules! __simd_munch {
    (@munch $chunk_size:expr, $unroll:expr, $idx:ident, [], [], [$( ($is_mut:ident $slice:expr , $ident:ident) )*], $body:tt, $pre:block, $post:block) => {
        __simd_exec!($chunk_size, $unroll, $idx, [$( ($is_mut $slice , $ident) ),*], $body, $pre, $post);
    };
    (@munch $chunk_size:expr, $unroll:expr, $idx:ident, [mut $slice:expr, $($slice_rest:tt)*], [$ident:ident, $($ident_rest:tt)*], [$( $acc:tt )*], $body:tt, $pre:block, $post:block) => {
        __simd_munch!(@munch $chunk_size, $unroll, $idx, [$($slice_rest)*], [$($ident_rest)*], [$( $acc )* (mut $slice , $ident)], $body, $pre, $post);
    };
    (@munch $chunk_size:expr, $unroll:expr, $idx:ident, [$slice:expr, $($slice_rest:tt)*], [$ident:ident, $($ident_rest:tt)*], [$( $acc:tt )*], $body:tt, $pre:block, $post:block) => {
        __simd_munch!(@munch $chunk_size, $unroll, $idx, [$($slice_rest)*], [$($ident_rest)*], [$( $acc )* (const $slice , $ident)], $body, $pre, $post);
    };
    (@munch $chunk_size:expr, $unroll:expr, $idx:ident, [mut $slice:expr], [$ident:ident], [$( $acc:tt )*], $body:tt, $pre:block, $post:block) => {
        __simd_munch!(@munch $chunk_size, $unroll, $idx, [], [], [$( $acc )* (mut $slice , $ident)], $body, $pre, $post);
    };
    (@munch $chunk_size:expr, $unroll:expr, $idx:ident, [$slice:expr], [$ident:ident], [$( $acc:tt )*], $body:tt, $pre:block, $post:block) => {
        __simd_munch!(@munch $chunk_size, $unroll, $idx, [], [], [$( $acc )* (const $slice , $ident)], $body, $pre, $post);
    };
}

#[macro_export]
macro_rules! __simd_dsl_munch {
    (@slices $unroll:expr, $chunk_size:expr, [$($slices:tt)*], as $($tail:tt)*) => {
        __simd_dsl_munch!(@idents $unroll, $chunk_size, [$($slices)*], [], $($tail)*);
    };
    (@slices $unroll:expr, $chunk_size:expr, [$($slices:tt)*], $head:tt $($tail:tt)*) => {
        __simd_dsl_munch!(@slices $unroll, $chunk_size, [$($slices)* $head], $($tail)*);
    };

    (@idents $unroll:expr, $chunk_size:expr, [$($slices:tt)*], [$($idents:tt)*], with index $idx:ident { $($body:tt)* }) => {
        __simd_munch!(@munch $chunk_size, $unroll, $idx, [$($slices)*], [$($idents)*], [], { $($body)* }, {}, {});
    };
    (@idents $unroll:expr, $chunk_size:expr, [$($slices:tt)*], [$($idents:tt)*], { $($body:tt)* }) => {
        __simd_munch!(@munch $chunk_size, $unroll, _internal_idx, [$($slices)*], [$($idents)*], [], { $($body)* }, {}, {});
    };
    (@idents $unroll:expr, $chunk_size:expr, [$($slices:tt)*], [$($idents:tt)*], $head:tt $($tail:tt)*) => {
        __simd_dsl_munch!(@idents $unroll, $chunk_size, [$($slices)*], [$($idents)* $head], $($tail)*);
    };
}

#[macro_export]
macro_rules! looping {
    ($unroll:tt unroll * $chunk_size:tt chunk for $($tail:tt)*) => {
        __simd_dsl_munch!(@slices $unroll, $chunk_size, [], $($tail)*);
    };
}
