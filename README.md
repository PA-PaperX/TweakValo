# TweakValo

TweakValo is an advanced, minimalist configuration utility designed specifically for VALORANT players. Built as a high-performance desktop application using **Tauri v2** and **React**, it provides an all-in-one window to manage graphics settings, apply True Stretch resolutions, and tweak NVIDIA driver profiles to optimize game performance and visibility.

---

## Features (ความสามารถของโปรแกรม)

1. **True Stretch Engine**
   ระบบปรับความละเอียดลบขอบดำ (True Stretch) ขั้นสูง ช่วยยืดเรโซลูชันเพื่อทำให้โมเดลตัวละครในเกมดูกว้างขึ้นอย่างมีประสิทธิภาพ พร้อมระบบแบ็กอัปและคืนค่า (Rollback) อัตโนมัติเมื่อเลิกใช้งาน
2. **NVIDIA Profile Tweaker**
   เชื่อมต่อและสั่งการ NVIDIA Profile Inspector แบบเบื้องหลัง เพื่อปรับลดคุณภาพกราฟิกระดับลึก (LOD Bias, Transparency Supersampling) รีดเฟรมเรตและลดควันให้มองเห็นชัดขึ้น
3. **Mature Content Restorer / Mod Manager**
   จัดการส่งไฟล์ดัดแปลงเนื้อหาสำหรับผู้ใหญ่ (เช่น เลือด, สัญลักษณ์เตือน) เข้าสู่โฟลเดอร์ของเกมให้โดยอัตโนมัติก่อนเริ่มเกม และจัดการลบออกให้เองเพื่อความปลอดภัย
4. **Smart Launch Integration**
   เชื่อมทุกการตั้งค่าเข้ากับปุ่ม "Play VALORANT" ในโปรแกรมตัวเดียว ตั้งค่าครั้งเดียว พอกดเปิดเกมปุ๊บ ระบบจะฉีดม็อด, ปรับ NVIDIA และยืด Stretch ให้พร้อมเล่นทันที


---

## Requirements & Warnings (คำเตือนและข้อควรระวัง)

> [!CAUTION]  
> **LEGAL & TOS WARNING (ความเสี่ยงเรื่องการแบนและข้อตกลง)**  
> โปรแกรมนี้ทำงานโดยใช้สคริปต์พื้นหลัง, การปรับ Priority CPU รัดคิว, การล้างแคช Standby Memory ใน Windows รวมถึงโหมดฉีดไฟล์ `.pak` (Mature Content Restorer) พฤติกรรมเหล่านี้อาจถูกตีความจากระบบ Vanguard Anti-Cheat ว่าเป็นการดัดแปลงโดยไม่ได้รับอนุญาต การเปิดใช้งาน **อาจส่งผลให้ไอดีและฮาร์ดแวร์ของคุณถูกระงับการใช้งานถาวร (HWID Ban)**  
> ทางผู้พัฒนาจะไม่รับผิดชอบใดๆ ทั้งสิ้นหากบัญชีผู้ใช้ของคุณได้รับผลกระทบ การกดใช้งานถือว่ารับรู้และรับความเสี่ยงด้วยตนเอง 100% (เพื่อความปลอดภัยที่สุด กรุณาอย่าเปิดใช้งานฟังก์ชัน Inject ไฟล์ลงตัวเกม)

- **System Requirements (ระบบที่รองรับ):** 
  - Windows 10 หรือ Windows 11
  - การ์ดจอ NVIDIA (จำเป็นสำหรับฟังก์ชัน Graphic Presets / LOD Bias)
  - ต้องมี VALORANT และ Riot Client ติดตั้งอยู่ในเครื่อง

---

## For Developers: How to Build & Contribute (สำหรับนักพัฒนา)

หากต้องการนำโปรเจกต์นี้ไปแก้ไข พัฒนาต่อ หรือ Build ใช้งานเอง จำเป็นต้องเตรียม Environment พื้นฐานให้พร้อม ดังนี้:

### 1. Prerequisites (สิ่งที่ต้องเตรียม)
- **Node.js**: เวอร์ชัน 24 ขึ้นไป
- **Rust**: เวอร์ชัน 1.85 ขึ้นไป (ดาวน์โหลดได้ผ่าน `rustup`)
- **Windows Build Tools**: ต้องลงเครื่องมือ "Desktop development with C++" ผ่าน Visual Studio Installer (เพื่อใช้ Windows 10/11 SDK ในการคอมไพล์โค้ดฝั่งระบบปฏิบัติการด้วย C++ MSVC)
- **Tauri CLI**: ติดตั้งผ่านคำสั่งฝั่งเว็บได้เลย

### 2. Setup Commands (คำสั่งสำหรับใช้งาน)
```bash
# 1. ติดตั้ง Dependencies ฝั่ง Frontend
npm install

# 2. เริ่มต้นเซิร์ฟเวอร์โหมด Development (Hot Reloading ของทั้ง React และ Rust)
npm run tauri dev

# 3. สร้างตัวติดตั้งโปรแกรม (Production Build -> จะได้ไฟล์ .exe เป็น NSIS Installer)
npm run tauri build
```
