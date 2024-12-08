import Image from 'next/image';
import KuRecImage from '../../../assets/images/KuRec-logo.webp';

export default function KuRecLogo() {
  return (
    <div style={{ width: '100%', maxWidth: '700px' }}>
      <Image
        src={KuRecImage}
        alt="KuRec Logo"
        layout="responsive"
        width={700}
        height={475}
      />
    </div>
  );
}
