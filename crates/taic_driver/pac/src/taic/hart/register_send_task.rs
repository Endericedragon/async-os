#[doc = "Register `register_send_task` reader"]
pub type R = crate::R<RegisterSendTaskSpec>;
#[doc = "Register `register_send_task` writer"]
pub type W = crate::W<RegisterSendTaskSpec>;
impl core::fmt::Debug for R {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.bits())
    }
}
impl W {}
#[doc = "Register send task.\n\nYou can [`read`](crate::Reg::read) this register and get [`register_send_task::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`register_send_task::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RegisterSendTaskSpec;
impl crate::RegisterSpec for RegisterSendTaskSpec {
    type Ux = u64;
}
#[doc = "`read()` method returns [`register_send_task::R`](R) reader structure"]
impl crate::Readable for RegisterSendTaskSpec {}
#[doc = "`write(|w| ..)` method takes [`register_send_task::W`](W) writer structure"]
impl crate::Writable for RegisterSendTaskSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u64 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u64 = 0;
}
#[doc = "`reset()` method sets register_send_task to value 0"]
impl crate::Resettable for RegisterSendTaskSpec {
    const RESET_VALUE: u64 = 0;
}
