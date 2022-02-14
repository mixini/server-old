/// The macro below basically allows any struct that is Serialize + Deserialize to be used as Redis args and values
/// Relies on bincode and is generally (and most hopefully) expected not to break
/// NOTE: If it does break, well... :sadface:
#[macro_export]
macro_rules! impl_redis_rv {
    ($( $t:ty ),+) => {
        $(
            impl redis::ToRedisArgs for $t {
                fn write_redis_args<W>(&self, out: &mut W)
                where
                    W: ?Sized + redis::RedisWrite,
                {
                    out.write_arg(&bincode::serialize(self).expect("Failed to bincode-serialize"));
                }
            }
            impl redis::FromRedisValue for $t {
                fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                    if let redis::Value::Data(data) = v {
                        let data_result = bincode::deserialize(data);
                        match data_result {
                            Ok(data) => Ok(data),
                            Err(_) => Err(redis::RedisError::from((
                                redis::ErrorKind::IoError,
                                "Response data not convertible",
                            ))),
                        }
                    } else {
                        Err(redis::RedisError::from((
                            redis::ErrorKind::TypeError,
                            "Response type not convertible",
                        )))
                    }
                }
            }
        )+
    };
}
