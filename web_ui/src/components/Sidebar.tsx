import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import styled from 'styled-components';
import { motion } from 'framer-motion';
import { 
  Home, 
  Bot, 
  Play, 
  GraduationCap, 
  Settings, 
  Activity,
  Zap
} from 'lucide-react';

const SidebarContainer = styled(motion.div)`
  position: fixed;
  left: 0;
  top: 0;
  height: 100vh;
  width: 240px;
  background: linear-gradient(180deg, #1a1a2e 0%, #16213e 100%);
  border-right: 1px solid #2d2d44;
  display: flex;
  flex-direction: column;
  z-index: 100;
  box-shadow: 4px 0 20px rgba(0, 0, 0, 0.3);
`;

const Logo = styled.div`
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 12px;
  border-bottom: 1px solid #2d2d44;
  background: rgba(79, 70, 229, 0.1);
`;

const LogoIcon = styled.div`
  width: 32px;
  height: 32px;
  background: linear-gradient(45deg, #4f46e5, #7c3aed);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
`;

const LogoText = styled.div`
  font-size: 18px;
  font-weight: 600;
  color: #e4e4e7;
`;

const Nav = styled.nav`
  flex: 1;
  padding: 20px 0;
`;

const NavItem = styled(Link)<{ $isActive: boolean }>`
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  color: ${props => props.$isActive ? '#4f46e5' : '#a1a1aa'};
  text-decoration: none;
  transition: all 0.2s ease;
  position: relative;
  margin: 2px 8px;
  border-radius: 8px;
  background: ${props => props.$isActive ? 'rgba(79, 70, 229, 0.1)' : 'transparent'};
  
  &:hover {
    background: rgba(79, 70, 229, 0.1);
    color: #4f46e5;
  }
  
  ${props => props.$isActive && `
    &::before {
      content: '';
      position: absolute;
      left: 0;
      top: 0;
      bottom: 0;
      width: 3px;
      background: linear-gradient(to bottom, #4f46e5, #7c3aed);
      border-radius: 0 2px 2px 0;
    }
  `}
`;

const NavIcon = styled.div`
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
`;

const StatusIndicator = styled.div`
  padding: 16px 20px;
  border-top: 1px solid #2d2d44;
  background: rgba(34, 197, 94, 0.1);
`;

const StatusDot = styled.div`
  width: 8px;
  height: 8px;
  background: #22c55e;
  border-radius: 50%;
  display: inline-block;
  margin-right: 8px;
  animation: pulse 2s infinite;
  
  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.5; }
    100% { opacity: 1; }
  }
`;

const menuItems = [
  { path: '/', icon: Home, label: 'Dashboard' },
  { path: '/agents', icon: Bot, label: 'Agents' },
  { path: '/arena', icon: Play, label: 'Arena' },
  { path: '/training', icon: GraduationCap, label: 'Training' },
  { path: '/settings', icon: Settings, label: 'Settings' },
];

export const Sidebar: React.FC = () => {
  const location = useLocation();

  return (
    <SidebarContainer
      initial={{ x: -240 }}
      animate={{ x: 0 }}
      transition={{ duration: 0.3 }}
    >
      <Logo>
        <LogoIcon>
          <Zap size={18} />
        </LogoIcon>
        <LogoText>Heaven AI</LogoText>
      </Logo>
      
      <Nav>
        {menuItems.map((item) => {
          const Icon = item.icon;
          const isActive = location.pathname === item.path;
          
          return (
            <NavItem 
              key={item.path} 
              to={item.path} 
              $isActive={isActive}
            >
              <NavIcon>
                <Icon size={20} />
              </NavIcon>
              {item.label}
            </NavItem>
          );
        })}
      </Nav>
      
      <StatusIndicator>
        <StatusDot />
        <span style={{ fontSize: '14px', color: '#22c55e' }}>
          System Online
        </span>
      </StatusIndicator>
    </SidebarContainer>
  );
};