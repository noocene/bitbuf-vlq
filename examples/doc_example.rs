use bitbuf::{BitBuf, BitBufMut};
use bitbuf_vlq::Vlq;

fn main() {
    // Create a buffer
    let mut data = [0u8; 8];

    // Very large number (requires 48 bits)
    let val: u64 = 25500050050;

    // Create a buffer handle to write into the array
    let mut buf = BitBufMut::new(&mut data);

    // Create a variable-length quantity (from any Into<u64>)
    let vlq: Vlq = Vlq::from(val);

    // Write the vlq data to the buffer
    buf.put_aligned(&*vlq).unwrap();

    // Note the length of the written data
    assert_eq!(buf.len(), 48);

    // Create a buffer to read the data back out
    let mut buf = BitBuf::new(&mut data);

    // Note the value is preserved
    assert_eq!(Vlq::read(&mut buf).unwrap(), val);

    // Use a smaller value
    let val: u64 = 20;

    // Create a new buffer handle to write into the array
    let mut buf = BitBufMut::new(&mut data);

    //. Create a new variable-length quantity
    let vlq: Vlq = Vlq::from(val);

    // Write the vlq data to the buffer
    buf.put_aligned(&*vlq).unwrap();

    // Note the shorter length of the written data
    assert_eq!(buf.len(), 16);

    // Create a buffer to read the data back out
    let mut buf = BitBuf::new(&mut data);

    // Note the value is preserved
    assert_eq!(Vlq::read(&mut buf).unwrap(), val);
}
