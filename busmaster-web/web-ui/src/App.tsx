import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { Box } from '@mui/material'
import { MainLayout } from '@/components/layout/MainLayout'
import { DashboardPage } from '@/pages/DashboardPage'
import { MessageWindowPage } from '@/pages/MessageWindowPage'
import { TransmitPage } from '@/pages/TransmitPage'
import { ConfigurationPage } from '@/pages/ConfigurationPage'

function App() {
  return (
    <Router>
      <MainLayout>
        <Routes>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/messages" element={<MessageWindowPage />} />
          <Route path="/transmit" element={<TransmitPage />} />
          <Route path="/configuration" element={<ConfigurationPage />} />
        </Routes>
      </MainLayout>
    </Router>
  )
}

export default App

