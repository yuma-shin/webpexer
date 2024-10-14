
export { Component as FormatSelector }

import { useTranslation } from 'react-i18next'
import { OutputFormat } from '../../preload/Types'

import {
  ToggleButtonGroup , ToggleButton ,
  FormControl , FormLabel , Box
} from '@mui/material'


interface Args {
  onChange : ( format : OutputFormat ) => void
  value : OutputFormat
}


function Component (
  { onChange , value } : Args
){

  const { t } = useTranslation('Conversion')

  return (
    <Box my = { 2 } >
      <FormControl>

        <FormLabel
          component = 'legend'
          children = { t('Format.Heading') }
        />

        <ToggleButtonGroup
          aria-label = 'Output Format'
          exclusive = { true }
          onChange = { ( _ , format ) => onChange(format) }
          color = 'primary'
          value = { value }
        >

          <ToggleButton
            children = { `WEBP` }
            value = 'webp'
          />

          <ToggleButton
            children = { `PNG` }
            value = 'png'
          />

        </ToggleButtonGroup>

      </FormControl>
    </Box>
  )
}
