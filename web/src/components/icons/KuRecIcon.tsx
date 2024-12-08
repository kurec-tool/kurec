import { Box, type BoxProps } from '@mui/material';
import Image from 'next/image';
import type React from 'react';
import KuRecIconImage from '../../../assets/images/KuRec-icon.webp';

type KuRecIconProps = {
  size: 'small' | 'large' | 'medium';
} & BoxProps;

const KuRecIcon: React.FC<KuRecIconProps> = ({ size, ...boxProps }) => {
  const width = size === 'small' ? 24 : size === 'medium' ? 36 : 48;
  const height = size === 'small' ? 24 : size === 'medium' ? 36 : 48;
  return (
    <Box {...boxProps}>
      <Image
        src={KuRecIconImage}
        alt="KuRec Icon"
        width={width}
        height={height}
      />
    </Box>
  );
};

export default KuRecIcon;
