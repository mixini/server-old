/// The macro below basically allows any struct that is Serialize + Deserialize to be used as Redis args and values
/// Relies on bincode and is generally (and most hopefully) expected not to break
/// NOTE: If it does break, well... :sadface:
#[macro_export]
macro_rules! impl_redis_rv {
    ($( $t:ty ),+) => {
        use redis::{
            ErrorKind as RedisErrorKind, FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs,
            Value as RedisValue,
        };
        $(
            impl ToRedisArgs for $t {
                fn write_redis_args<W>(&self, out: &mut W)
                where
                    W: ?Sized + RedisWrite,
                {
                    out.write_arg(&bincode::serialize(self).expect("Failed to bincode-serialize"));
                }
            }
            impl FromRedisValue for $t {
                fn from_redis_value(v: &RedisValue) -> RedisResult<Self> {
                    if let RedisValue::Data(data) = v {
                        let data_result = bincode::deserialize(data);
                        match data_result {
                            Ok(data) => Ok(data),
                            Err(_) => Err(RedisError::from((
                                RedisErrorKind::IoError,
                                "Response data not convertible",
                            ))),
                        }
                    } else {
                        Err(RedisError::from((
                            RedisErrorKind::TypeError,
                            "Response type not convertible",
                        )))
                    }
                }
            }
        )+
    };
}
