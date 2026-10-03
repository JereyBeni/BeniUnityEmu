//! Internal JNI representation (no real JVM).
//! Only the subset old Unity games typically need.

#[derive(Debug, Clone, Copy)]
pub struct JClass(pub u64);
#[derive(Debug, Clone, Copy)]
pub struct JObject(pub u64);
#[derive(Debug, Clone, Copy)]
pub struct JMethodID(pub u64);
#[derive(Debug, Clone, Copy)]
pub struct JFieldID(pub u64);
#[derive(Debug, Clone)]
pub struct JString(pub String);

pub struct JniEnv {
    next_id: u64,
}

impl JniEnv {
    pub fn new() -> Self {
        JniEnv { next_id: 1 }
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn find_class(&mut self, name: &str) -> Result<JClass, String> {
        println!("[JNI] FindClass(\"{}\") – stub", name);
        Ok(JClass(self.alloc_id()))
    }

    pub fn get_method_id(&mut self, _cls: JClass, name: &str, sig: &str) -> Result<JMethodID, String> {
        println!("[JNI] GetMethodID(\"{}\", \"{}\") – stub", name, sig);
        Ok(JMethodID(self.alloc_id()))
    }

    pub fn get_static_method_id(&mut self, _cls: JClass, name: &str, sig: &str) -> Result<JMethodID, String> {
        println!("[JNI] GetStaticMethodID(\"{}\", \"{}\") – stub", name, sig);
        Ok(JMethodID(self.alloc_id()))
    }

    pub fn call_void_method(&mut self, _obj: JObject, _mid: JMethodID) -> Result<(), String> {
        println!("[JNI] CallVoidMethod – stub");
        Ok(())
    }

    pub fn call_int_method(&mut self, _obj: JObject, _mid: JMethodID) -> Result<i32, String> {
        println!("[JNI] CallIntMethod – stub");
        Ok(0)
    }

    pub fn call_object_method(&mut self, _obj: JObject, _mid: JMethodID) -> Result<JObject, String> {
        println!("[JNI] CallObjectMethod – stub");
        Ok(JObject(self.alloc_id()))
    }

    pub fn new_string_utf(&mut self, s: &str) -> Result<JString, String> {
        Ok(JString(s.to_string()))
    }
}

impl Default for JniEnv {
    fn default() -> Self { Self::new() }
}
