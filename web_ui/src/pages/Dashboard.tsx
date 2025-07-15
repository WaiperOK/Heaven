import React from 'react';
import styled from 'styled-components';
import { motion } from 'framer-motion';
import { Activity, Bot, Zap, TrendingUp } from 'lucide-react';

const DashboardContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 24px;
`;

const HeaderSection = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
`;

const Title = styled.h1`
  font-size: 32px;
  font-weight: 700;
  color: #e4e4e7;
  margin: 0;
`;

const MetricsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
`;

const MetricCard = styled(motion.div)`
  background: linear-gradient(135deg, #1e1e2e 0%, #2d2d44 100%);
  border: 1px solid #3f3f46;
  border-radius: 12px;
  padding: 24px;
  display: flex;
  align-items: center;
  gap: 16px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
`;

const MetricIcon = styled.div<{ color: string }>`
  width: 48px;
  height: 48px;
  background: ${props => props.color};
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
`;

const MetricInfo = styled.div`
  flex: 1;
`;

const MetricValue = styled.div`
  font-size: 24px;
  font-weight: 700;
  color: #e4e4e7;
  margin-bottom: 4px;
`;

const MetricLabel = styled.div`
  font-size: 14px;
  color: #a1a1aa;
`;

const StatusGrid = styled.div`
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
`;

const StatusCard = styled(motion.div)`
  background: linear-gradient(135deg, #1e1e2e 0%, #2d2d44 100%);
  border: 1px solid #3f3f46;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
`;

const StatusTitle = styled.h3`
  font-size: 18px;
  font-weight: 600;
  color: #e4e4e7;
  margin-bottom: 16px;
`;

const StatusItem = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #3f3f46;
  
  &:last-child {
    border-bottom: none;
  }
`;

const StatusLabel = styled.span`
  color: #a1a1aa;
  font-size: 14px;
`;

const StatusValue = styled.span<{ status: 'online' | 'offline' | 'warning' }>`
  font-size: 14px;
  font-weight: 500;
  color: ${props => {
    switch (props.status) {
      case 'online': return '#22c55e';
      case 'offline': return '#ef4444';
      case 'warning': return '#f59e0b';
      default: return '#a1a1aa';
    }
  }};
`;

export const Dashboard: React.FC = () => {
  return (
    <DashboardContainer>
      <HeaderSection>
        <Title>Dashboard</Title>
      </HeaderSection>
      
      <MetricsGrid>
        <MetricCard
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
        >
          <MetricIcon color="linear-gradient(45deg, #4f46e5, #7c3aed)">
            <Bot size={24} />
          </MetricIcon>
          <MetricInfo>
            <MetricValue>12</MetricValue>
            <MetricLabel>Active Agents</MetricLabel>
          </MetricInfo>
        </MetricCard>
        
        <MetricCard
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
        >
          <MetricIcon color="linear-gradient(45deg, #22c55e, #16a34a)">
            <Activity size={24} />
          </MetricIcon>
          <MetricInfo>
            <MetricValue>248</MetricValue>
            <MetricLabel>Matches Completed</MetricLabel>
          </MetricInfo>
        </MetricCard>
        
        <MetricCard
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
        >
          <MetricIcon color="linear-gradient(45deg, #f59e0b, #d97706)">
            <Zap size={24} />
          </MetricIcon>
          <MetricInfo>
            <MetricValue>89%</MetricValue>
            <MetricLabel>System Performance</MetricLabel>
          </MetricInfo>
        </MetricCard>
        
        <MetricCard
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
        >
          <MetricIcon color="linear-gradient(45deg, #06b6d4, #0891b2)">
            <TrendingUp size={24} />
          </MetricIcon>
          <MetricInfo>
            <MetricValue>3</MetricValue>
            <MetricLabel>Training Jobs</MetricLabel>
          </MetricInfo>
        </MetricCard>
      </MetricsGrid>
      
      <StatusGrid>
        <StatusCard
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.5 }}
        >
          <StatusTitle>System Status</StatusTitle>
          <StatusItem>
            <StatusLabel>Arena Core</StatusLabel>
            <StatusValue status="online">Online</StatusValue>
          </StatusItem>
          <StatusItem>
            <StatusLabel>LLM Service</StatusLabel>
            <StatusValue status="online">Online</StatusValue>
          </StatusItem>
          <StatusItem>
            <StatusLabel>Database</StatusLabel>
            <StatusValue status="online">Online</StatusValue>
          </StatusItem>
          <StatusItem>
            <StatusLabel>Ollama</StatusLabel>
            <StatusValue status="online">Online</StatusValue>
          </StatusItem>
        </StatusCard>
        
        <StatusCard
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.6 }}
        >
          <StatusTitle>Recent Activity</StatusTitle>
          <StatusItem>
            <StatusLabel>Agent "Gladiator-1" trained</StatusLabel>
            <StatusValue status="online">2 min ago</StatusValue>
          </StatusItem>
          <StatusItem>
            <StatusLabel>Match completed</StatusLabel>
            <StatusValue status="online">5 min ago</StatusValue>
          </StatusItem>
          <StatusItem>
            <StatusLabel>New agent created</StatusLabel>
            <StatusValue status="online">12 min ago</StatusValue>
          </StatusItem>
          <StatusItem>
            <StatusLabel>System backup</StatusLabel>
            <StatusValue status="online">1 hour ago</StatusValue>
          </StatusItem>
        </StatusCard>
      </StatusGrid>
    </DashboardContainer>
  );
};